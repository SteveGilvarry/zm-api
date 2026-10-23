//! RTSP requests and responses (RFC 2326), and the URLs this server answers.
//!
//! Requests are read with hard limits: a client is untrusted until its
//! signed link checks out, so nothing it sends can make the server buffer
//! more than a few kilobytes.

use std::io;

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};
use url::Url;

/// Most bytes a request line plus headers may take.
pub const MAX_HEAD: usize = 8 * 1024;
/// Most bytes of a request body (`GET_PARAMETER` keep-alives carry a few).
pub const MAX_BODY: usize = 4 * 1024;
/// Most headers a request may have.
pub const MAX_HEADERS: usize = 32;

/// One RTSP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl Request {
    /// A header's value, by case-insensitive name.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// What arrived on the connection.
#[derive(Debug, PartialEq, Eq)]
pub enum Incoming {
    Request(Request),
    /// Interleaved binary data from the client (its RTCP reports), skipped.
    Data,
}

fn invalid(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.to_string())
}

/// Read the next request or interleaved frame. `None` at a clean end of
/// stream.
pub async fn read<R: AsyncBufRead + Unpin>(r: &mut R) -> io::Result<Option<Incoming>> {
    let first = match r.fill_buf().await?.first() {
        Some(b) => *b,
        None => return Ok(None),
    };
    if first == b'$' {
        let mut head = [0u8; 4];
        r.read_exact(&mut head).await?;
        let len = u16::from_be_bytes([head[2], head[3]]) as u64;
        let skipped = tokio::io::copy(&mut r.take(len), &mut tokio::io::sink()).await?;
        if skipped != len {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        return Ok(Some(Incoming::Data));
    }

    let mut lines = Vec::new();
    let mut used = 0usize;
    loop {
        let mut line = Vec::new();
        let n = r
            .take((MAX_HEAD - used) as u64)
            .read_until(b'\n', &mut line)
            .await?;
        if n == 0 {
            return if lines.is_empty() {
                Ok(None)
            } else {
                Err(io::ErrorKind::UnexpectedEof.into())
            };
        }
        used += n;
        if line.last() != Some(&b'\n') {
            return Err(invalid("request head too long"));
        }
        let text = String::from_utf8(line).map_err(|_| invalid("request is not UTF-8"))?;
        let text = text.trim_end_matches(['\r', '\n']);
        if text.is_empty() {
            if lines.is_empty() {
                continue; // stray CRLF between requests
            }
            break;
        }
        if lines.len() > MAX_HEADERS {
            return Err(invalid("too many headers"));
        }
        lines.push(text.to_string());
    }

    let mut parts = lines[0].split(' ');
    let (Some(method), Some(url), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(invalid("bad request line"));
    };
    if !version.starts_with("RTSP/1.") {
        return Err(invalid("not RTSP/1.x"));
    }
    let mut headers = Vec::new();
    for line in &lines[1..] {
        let (k, v) = line.split_once(':').ok_or_else(|| invalid("bad header"))?;
        headers.push((k.trim().to_string(), v.trim().to_string()));
    }
    let req = Request {
        method: method.to_string(),
        url: url.to_string(),
        headers,
    };
    let body = match req.header("Content-Length") {
        Some(v) => v
            .parse::<usize>()
            .map_err(|_| invalid("bad Content-Length"))?,
        None => 0,
    };
    if body > MAX_BODY {
        return Err(invalid("body too large"));
    }
    if body > 0 {
        let mut skip = vec![0u8; body];
        r.read_exact(&mut skip).await?;
    }
    Ok(Some(Incoming::Request(req)))
}

/// An RTSP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn header(mut self, name: &str, value: impl Into<String>) -> Self {
        self.headers.push((name.to_string(), value.into()));
        self
    }

    pub fn body(mut self, content_type: &str, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self.header("Content-Type", content_type)
    }

    /// The response on the wire, echoing the request's `CSeq`.
    pub fn encode(&self, cseq: Option<&str>) -> Vec<u8> {
        let mut out = format!("RTSP/1.0 {} {}\r\n", self.status, reason(self.status));
        if let Some(cseq) = cseq {
            out.push_str(&format!("CSeq: {cseq}\r\n"));
        }
        out.push_str("Server: zm-api\r\n");
        for (k, v) in &self.headers {
            out.push_str(&format!("{k}: {v}\r\n"));
        }
        if !self.body.is_empty() {
            out.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        }
        out.push_str("\r\n");
        let mut bytes = out.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        454 => "Session Not Found",
        455 => "Method Not Valid in This State",
        459 => "Aggregate Operation Not Allowed",
        461 => "Unsupported Transport",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

/// The query parameter that carries the signed link.
pub const TOKEN_PARAM: &str = "st";

/// What a request URL names: `rtsp://host[:port]/live/{monitor_id}` for the
/// session, with `/track0` for the video stream, and the signed link in
/// `?st=`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub monitor_id: u32,
    pub track: bool,
    pub token: Option<String>,
    /// `scheme://host[:port]`, to build the stream's control URL.
    pub origin: String,
}

pub fn parse_target(raw: &str) -> Option<Target> {
    let url = Url::parse(raw).ok()?;
    if !matches!(url.scheme(), "rtsp" | "rtsps") {
        return None;
    }
    let mut segments = url.path_segments()?.filter(|s| !s.is_empty());
    if segments.next()? != "live" {
        return None;
    }
    let id = segments.next()?;
    if !id.bytes().all(|b| b.is_ascii_digit()) || id.len() > 10 {
        return None;
    }
    let monitor_id = id.parse().ok()?;
    let track = match segments.next() {
        None => false,
        Some("track0") => true,
        Some(_) => return None,
    };
    if segments.next().is_some() {
        return None;
    }
    let token = url
        .query_pairs()
        .find(|(k, _)| k == TOKEN_PARAM)
        .map(|(_, v)| v.into_owned())
        .filter(|v| !v.is_empty());
    let origin = match url.port() {
        Some(p) => format!("{}://{}:{p}", url.scheme(), url.host_str()?),
        None => format!("{}://{}", url.scheme(), url.host_str()?),
    };
    Some(Target {
        monitor_id,
        track,
        token,
        origin,
    })
}

/// The control URL of a monitor's video stream, carrying the signed link so
/// clients that resolve it on their own still send it.
pub fn track_url(origin: &str, monitor_id: u32, token: &str) -> String {
    format!(
        "{origin}/live/{monitor_id}/track0?{TOKEN_PARAM}={}",
        urlencoding::encode(token)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    async fn read_all(input: &[u8]) -> Vec<io::Result<Option<Incoming>>> {
        let mut r = BufReader::new(input);
        let mut out = Vec::new();
        loop {
            let item = read(&mut r).await;
            let done = !matches!(item, Ok(Some(_)));
            out.push(item);
            if done {
                return out;
            }
        }
    }

    #[tokio::test]
    async fn reads_requests_and_skips_interleaved_data_and_bodies() {
        let input = b"OPTIONS rtsp://h/live/1 RTSP/1.0\r\nCSeq: 1\r\n\r\n\
            $\x01\x00\x03abc\
            GET_PARAMETER rtsp://h/live/1 RTSP/1.0\r\nCSeq: 2\r\nContent-Length: 4\r\n\r\nping";
        let items = read_all(input).await;
        let Ok(Some(Incoming::Request(a))) = &items[0] else {
            panic!("{items:?}")
        };
        assert_eq!(a.method, "OPTIONS");
        assert_eq!(a.header("cseq"), Some("1"));
        assert!(matches!(items[1], Ok(Some(Incoming::Data))));
        let Ok(Some(Incoming::Request(b))) = &items[2] else {
            panic!()
        };
        assert_eq!(b.method, "GET_PARAMETER");
        assert!(matches!(items[3], Ok(None)));
    }

    #[tokio::test]
    async fn oversized_or_malformed_requests_are_errors() {
        let long = format!("OPTIONS rtsp://h/{} RTSP/1.0\r\n\r\n", "a".repeat(MAX_HEAD));
        assert!(read_all(long.as_bytes()).await[0].is_err());
        let many = format!(
            "OPTIONS * RTSP/1.0\r\n{}\r\n",
            "X: y\r\n".repeat(MAX_HEADERS + 2)
        );
        assert!(read_all(many.as_bytes()).await[0].is_err());
        let body = b"SET_PARAMETER * RTSP/1.0\r\nContent-Length: 999999\r\n\r\n";
        assert!(read_all(body).await[0].is_err());
        assert!(read_all(b"GET / HTTP/1.1\r\n\r\n").await[0].is_err());
        assert!(read_all(b"OPTIONS * RTSP/1.0\r\nnocolon\r\n\r\n").await[0].is_err());
    }

    #[test]
    fn responses_echo_cseq_and_length() {
        let r = Response::new(200)
            .header("Public", "OPTIONS")
            .body("application/sdp", "v=0\r\n");
        let text = String::from_utf8(r.encode(Some("7"))).unwrap();
        assert!(text.starts_with("RTSP/1.0 200 OK\r\nCSeq: 7\r\n"));
        assert!(text.contains("Content-Length: 5\r\n\r\nv=0\r\n"));
    }

    #[test]
    fn targets() {
        let t = parse_target("rtsp://cam.local:8554/live/12?st=a%2Bb").unwrap();
        assert_eq!(t.monitor_id, 12);
        assert!(!t.track);
        assert_eq!(t.token.as_deref(), Some("a+b"));
        assert_eq!(t.origin, "rtsp://cam.local:8554");
        let t = parse_target("rtsps://10.0.0.1/live/3/track0/").unwrap();
        assert!(t.track);
        assert_eq!(t.token, None);
        assert_eq!(t.origin, "rtsps://10.0.0.1");
        for bad in [
            "http://h/live/1",
            "rtsp://h/live/x",
            "rtsp://h/live/1/track1",
            "rtsp://h/live/1/track0/x",
            "rtsp://h/other/1",
            "rtsp://h/live/99999999999",
            "*",
        ] {
            assert!(parse_target(bad).is_none(), "{bad}");
        }
        assert_eq!(
            track_url("rtsp://h:1", 4, "a+b"),
            "rtsp://h:1/live/4/track0?st=a%2Bb"
        );
    }
}
