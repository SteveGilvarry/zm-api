//! HAP accessory HTTP server (HAP spec ch. 6.7).
//!
//! HAP runs an HTTP/1.1-shaped protocol over TCP. Before pairing, `/pair-setup`
//! and `/pair-verify` are exchanged in cleartext; once Pair-Verify completes the
//! connection switches to ChaCha20-Poly1305 framing ([`SessionCrypto`]) and all
//! further requests (`/accessories`, `/characteristics`, `/pairings`,
//! `/resource`) are encrypted.
//!
//! This module owns the per-connection loop, a minimal request parser, and the
//! route handlers. The accessory/camera/crypto logic lives in sibling modules;
//! here we glue them to the socket.

use std::net::IpAddr;
use std::sync::Arc;

use base64::Engine as _;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

use super::accessory::{iid, CAMERA_AID};
use super::pairing::setup::PairSetupSession;
use super::pairing::verify::PairVerifySession;
use super::session::SessionCrypto;
use super::tlv8::{TlvReader, TlvType, TlvWriter};
use super::HomeKitServer;

const CT_TLV8: &str = "application/pairing+tlv8";
const CT_JSON: &str = "application/hap+json";
const CT_JPEG: &str = "image/jpeg";

/// A parsed HTTP request.
struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}

/// Try to parse one HTTP/1.1 request from `buf`. Returns the request and the
/// number of bytes consumed, or `None` if more data is needed.
fn parse_request(buf: &[u8]) -> Option<(Request, usize)> {
    let header_end = find_subsequence(buf, b"\r\n\r\n")? + 4;
    let head = std::str::from_utf8(&buf[..header_end]).ok()?;
    let mut lines = head.split("\r\n");
    let request_line = lines.next()?;
    let mut parts = request_line.split(' ');
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();

    let mut content_length = 0usize;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().ok()?;
            }
        }
    }

    let total = header_end + content_length;
    if buf.len() < total {
        return None;
    }
    let body = buf[header_end..total].to_vec();
    Some((Request { method, path, body }, total))
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Build an HTTP/1.1 response.
fn response(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n",
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(body);
    out
}

/// A 204 No Content response (used for successful characteristic writes).
fn no_content() -> Vec<u8> {
    b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n".to_vec()
}

/// Outcome of handling one request: the bytes to send and an optional session
/// to activate *after* sending (Pair-Verify M4).
struct Handled {
    response: Vec<u8>,
    activate: Option<SessionCrypto>,
}

/// Per-connection mutable pairing state.
#[derive(Default)]
struct ConnState {
    setup: PairSetupSession,
    verify: PairVerifySession,
}

/// Handle a single accepted TCP connection for its lifetime.
pub async fn serve_connection(server: Arc<HomeKitServer>, mut stream: TcpStream) {
    let peer = stream.peer_addr().ok();
    let accessory_ip: IpAddr = stream
        .local_addr()
        .map(|a| a.ip())
        .unwrap_or(IpAddr::from([0, 0, 0, 0]));

    let mut conn = ConnState::default();
    let mut session: Option<SessionCrypto> = None;
    let mut raw: Vec<u8> = Vec::new();
    let mut plain: Vec<u8> = Vec::new();
    let mut read_buf = [0u8; 8192];

    loop {
        let n = match stream.read(&mut read_buf).await {
            Ok(0) => break, // peer closed
            Ok(n) => n,
            Err(e) => {
                debug!(?peer, "homekit read error: {e}");
                break;
            }
        };
        raw.extend_from_slice(&read_buf[..n]);

        // Move available bytes into the plaintext buffer, decrypting if the
        // session is active.
        if let Some(sess) = session.as_mut() {
            match sess.decrypt_available(&raw) {
                Ok((pt, consumed)) => {
                    raw.drain(..consumed);
                    plain.extend_from_slice(&pt);
                }
                Err(e) => {
                    warn!(?peer, "homekit session decrypt failed: {e}");
                    break;
                }
            }
        } else {
            plain.append(&mut raw);
        }

        // Process every complete request currently buffered.
        while let Some((req, consumed)) = parse_request(&plain) {
            let handled = handle_request(&server, &mut conn, accessory_ip, &req).await;
            let bytes = if let Some(sess) = session.as_mut() {
                sess.encrypt(&handled.response)
            } else {
                handled.response
            };
            if let Err(e) = stream.write_all(&bytes).await {
                debug!(?peer, "homekit write error: {e}");
                return;
            }
            plain.drain(..consumed);
            // Activate encryption AFTER the (cleartext) Pair-Verify M4 reply.
            if let Some(new_session) = handled.activate {
                session = Some(new_session);
            }
        }
    }
}

async fn handle_request(
    server: &Arc<HomeKitServer>,
    conn: &mut ConnState,
    accessory_ip: IpAddr,
    req: &Request,
) -> Handled {
    let path = req.path.split('?').next().unwrap_or(&req.path);
    match (req.method.as_str(), path) {
        ("POST", "/pair-setup") => {
            let body = conn
                .setup
                .handle(&req.body, &server.store, &server.config.pin_digits());
            // Reflect paired state in mDNS once setup completes.
            if let Some(adv) = server.advertiser().as_ref() {
                let _ = adv.set_paired(server.store.is_paired());
            }
            Handled {
                response: response("200 OK", CT_TLV8, &body),
                activate: None,
            }
        }
        ("POST", "/pair-verify") => {
            let result = conn.verify.handle(&req.body, &server.store);
            Handled {
                response: response("200 OK", CT_TLV8, &result.response),
                activate: result.session,
            }
        }
        ("GET", "/accessories") => Handled {
            response: response(
                "200 OK",
                CT_JSON,
                server.accessories_json().to_string().as_bytes(),
            ),
            activate: None,
        },
        ("GET", "/characteristics") => Handled {
            response: handle_get_characteristics(server, &req.path),
            activate: None,
        },
        ("PUT", "/characteristics") => Handled {
            response: handle_put_characteristics(server, accessory_ip, &req.body).await,
            activate: None,
        },
        ("POST", "/pairings") => Handled {
            response: response("200 OK", CT_TLV8, &handle_pairings(server, &req.body)),
            activate: None,
        },
        ("POST", "/resource") => Handled {
            response: handle_resource(server, &req.body).await,
            activate: None,
        },
        _ => Handled {
            response: response("404 Not Found", CT_JSON, b"{}"),
            activate: None,
        },
    }
}

/// `GET /characteristics?id=aid.iid,aid.iid` — read characteristic values.
fn handle_get_characteristics(server: &Arc<HomeKitServer>, full_path: &str) -> Vec<u8> {
    let ids = full_path
        .split_once("id=")
        .map(|(_, q)| q.split('&').next().unwrap_or(q))
        .unwrap_or("");

    let mut chars = Vec::new();
    for pair in ids.split(',').filter(|s| !s.is_empty()) {
        let Some((aid, ch_iid)) = pair.split_once('.') else {
            continue;
        };
        let (aid, ch_iid) = (aid.parse::<u64>().ok(), ch_iid.parse::<u64>().ok());
        let (Some(aid), Some(ch_iid)) = (aid, ch_iid) else {
            continue;
        };
        let value = server.read_characteristic(aid, ch_iid);
        chars.push(serde_json::json!({ "aid": aid, "iid": ch_iid, "value": value }));
    }
    let body = serde_json::json!({ "characteristics": chars });
    response("200 OK", CT_JSON, body.to_string().as_bytes())
}

/// `PUT /characteristics` — write characteristic values (camera negotiation).
async fn handle_put_characteristics(
    server: &Arc<HomeKitServer>,
    accessory_ip: IpAddr,
    body: &[u8],
) -> Vec<u8> {
    let Ok(json): Result<Value, _> = serde_json::from_slice(body) else {
        return response("400 Bad Request", CT_JSON, b"{}");
    };
    let Some(items) = json["characteristics"].as_array() else {
        return no_content();
    };

    for item in items {
        let aid = item["aid"].as_u64().unwrap_or(0);
        let ch_iid = item["iid"].as_u64().unwrap_or(0);
        if aid != CAMERA_AID {
            continue;
        }
        if let Some(value_b64) = item["value"].as_str() {
            let Ok(tlv) = base64::engine::general_purpose::STANDARD.decode(value_b64) else {
                continue;
            };
            match ch_iid {
                iid::CAMERA_SETUP_ENDPOINTS => {
                    server.handle_setup_endpoints(accessory_ip, &tlv).await;
                }
                iid::CAMERA_SELECTED_RTP => {
                    server.handle_selected_rtp(&tlv).await;
                }
                _ => {}
            }
        }
    }
    no_content()
}

/// `POST /pairings` — add/remove/list pairings (HAP spec 5.10–5.12).
fn handle_pairings(server: &Arc<HomeKitServer>, body: &[u8]) -> Vec<u8> {
    let Some(r) = TlvReader::parse(body) else {
        return error_state(2);
    };
    // Method: 3 = add pairing, 4 = remove pairing, 5 = list pairings.
    match r.get_u8(TlvType::Method) {
        Some(4) => {
            if let Some(id) = r.get(TlvType::Identifier) {
                let id = String::from_utf8_lossy(id).into_owned();
                let _ = server.store.remove_controller(&id);
                if let Some(adv) = server.advertiser().as_ref() {
                    let _ = adv.set_paired(server.store.is_paired());
                }
            }
            let mut w = TlvWriter::new();
            w.push_u8(TlvType::State, 2);
            w.into_bytes()
        }
        _ => {
            // Add/list are acknowledged with a minimal success for Phase 1.
            let mut w = TlvWriter::new();
            w.push_u8(TlvType::State, 2);
            w.into_bytes()
        }
    }
}

/// `POST /resource` — return a JPEG snapshot for the camera.
async fn handle_resource(server: &Arc<HomeKitServer>, _body: &[u8]) -> Vec<u8> {
    match server.snapshot().as_ref() {
        Some(svc) => match svc.get_snapshot(server.config.monitor_id).await {
            Ok(jpeg) => {
                info!(
                    monitor = server.config.monitor_id,
                    bytes = jpeg.len(),
                    "homekit snapshot"
                );
                response("200 OK", CT_JPEG, &jpeg)
            }
            Err(e) => {
                warn!("homekit snapshot failed: {e}");
                response("500 Internal Server Error", CT_JSON, b"{}")
            }
        },
        None => response("503 Service Unavailable", CT_JSON, b"{}"),
    }
}

fn error_state(state: u8) -> Vec<u8> {
    let mut w = TlvWriter::new();
    w.push_u8(TlvType::State, state)
        .push_u8(TlvType::Error, super::tlv8::TlvError::Unknown as u8);
    w.into_bytes()
}
