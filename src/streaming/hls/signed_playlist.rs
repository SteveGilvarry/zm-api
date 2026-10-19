//! Sign the URIs a playlist lists, for players that authenticate in the URL.
//!
//! A player that can't send an `Authorization` header (FFmpeg, and so Home
//! Assistant; VLC; Safari's native HLS) requests every URI a playlist lists
//! exactly as written, so a playlist with bare URIs gets it the playlist and
//! then a 401 on every segment (#153). Each listed URI is given a signed
//! media URL instead (see [`crate::util::media_url`]):
//!
//! - child playlists (`*.m3u8`, refetched for as long as the player plays)
//!   carry the same long-lived credential the playlist was fetched with;
//! - everything else (init, segments, LL-HLS parts) carries a fresh
//!   short-lived signature, renewed every time the playlist is refetched.
//!
//! Header-authenticated requests carry no URL credential and get the
//! playlist unchanged.

use crate::util::media_url;

/// The query parameters to attach to a playlist's URIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriSigner {
    /// `st` value for child playlists.
    pub playlist: String,
    /// `st` value for media URIs.
    pub media: String,
}

/// Attach the signer's parameters to each URI line and `URI="…"` attribute.
pub fn sign_playlist(playlist: &str, signer: &UriSigner) -> String {
    let mut out = String::with_capacity(playlist.len() + 128);
    for line in playlist.split_inclusive('\n') {
        let (body, eol) = match line.strip_suffix('\n') {
            Some(b) => match b.strip_suffix('\r') {
                Some(b) => (b, "\r\n"),
                None => (b, "\n"),
            },
            None => (line, ""),
        };
        if body.is_empty() {
            out.push_str(line);
            continue;
        }
        if body.starts_with('#') {
            out.push_str(&sign_tag(body, signer));
        } else {
            out.push_str(&sign_uri(body, signer));
        }
        out.push_str(eol);
    }
    out
}

fn sign_tag(line: &str, signer: &UriSigner) -> String {
    const ATTR: &str = "URI=\"";
    let mut out = String::with_capacity(line.len() + 96);
    let mut rest = line;
    while let Some(start) = rest.find(ATTR) {
        let value_start = start + ATTR.len();
        let Some(len) = rest[value_start..].find('"') else {
            break;
        };
        out.push_str(&rest[..value_start]);
        out.push_str(&sign_uri(&rest[value_start..value_start + len], signer));
        rest = &rest[value_start + len..];
    }
    out.push_str(rest);
    out
}

fn sign_uri(uri: &str, signer: &UriSigner) -> String {
    let path = uri.split(['?', '#']).next().unwrap_or(uri);
    let value = if path.ends_with(".m3u8") {
        &signer.playlist
    } else {
        &signer.media
    };
    let sep = if uri.contains('?') { '&' } else { '?' };
    format!(
        "{uri}{sep}{}={}",
        media_url::PARAM,
        urlencoding::encode(value)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signer() -> UriSigner {
        UriSigner {
            playlist: "LONG".into(),
            media: "SHORT".into(),
        }
    }

    #[test]
    fn segments_get_the_short_signature_and_child_playlists_the_long_one() {
        let media = "#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:2.000,\nsegment_1.m4s\n\
                     #EXTINF:2.000,\nsegment/2.m4s?x=1\n";
        let out = sign_playlist(media, &signer());
        assert!(out.contains("#EXT-X-MAP:URI=\"init.mp4?st=SHORT\"\n"));
        assert!(out.contains("\nsegment_1.m4s?st=SHORT\n"));
        assert!(out.contains("\nsegment/2.m4s?x=1&st=SHORT\n"));

        let master = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\nlive.m3u8\n";
        assert!(sign_playlist(master, &signer()).ends_with("\nlive.m3u8?st=LONG\n"));
    }

    #[test]
    fn ll_hls_parts_and_preload_hints_are_signed() {
        let part = "#EXT-X-PART:DURATION=0.5,URI=\"part_1.m4s\",INDEPENDENT=YES\n";
        assert_eq!(
            sign_playlist(part, &signer()),
            "#EXT-X-PART:DURATION=0.5,URI=\"part_1.m4s?st=SHORT\",INDEPENDENT=YES\n"
        );
        let hint = "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part_2.m4s\"\n";
        assert!(sign_playlist(hint, &signer()).contains("URI=\"part_2.m4s?st=SHORT\""));
    }

    #[test]
    fn values_are_percent_encoded_and_crlf_kept() {
        let s = UriSigner {
            playlist: "a b".into(),
            media: "c&d".into(),
        };
        assert_eq!(
            sign_playlist("#EXTM3U\r\nseg.m4s\r\n", &s),
            "#EXTM3U\r\nseg.m4s?st=c%26d\r\n"
        );
    }
}
