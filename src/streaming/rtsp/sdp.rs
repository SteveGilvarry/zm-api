//! The session description a `DESCRIBE` returns: one video stream, with the
//! parameter sets from the monitor's cached keyframe so a player can set up
//! its decoder before the first frame.

use base64::{engine::general_purpose::STANDARD, Engine as _};

use super::rtp::{nal_units, PAYLOAD_TYPE};
use crate::streaming::source::media::VideoCodec;

/// Parameter sets found in an access unit.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParameterSets<'a> {
    pub vps: Option<&'a [u8]>,
    pub sps: Option<&'a [u8]>,
    pub pps: Option<&'a [u8]>,
}

/// The first VPS, SPS and PPS in an Annex B access unit.
pub fn parameter_sets(codec: VideoCodec, access_unit: &[u8]) -> ParameterSets<'_> {
    let mut sets = ParameterSets::default();
    for nal in nal_units(access_unit) {
        let slot = match codec {
            VideoCodec::H265 => match (nal[0] >> 1) & 0x3F {
                32 => &mut sets.vps,
                33 => &mut sets.sps,
                34 => &mut sets.pps,
                _ => continue,
            },
            VideoCodec::H264 | VideoCodec::Unknown => match nal[0] & 0x1F {
                7 => &mut sets.sps,
                8 => &mut sets.pps,
                _ => continue,
            },
        };
        slot.get_or_insert(nal);
    }
    sets
}

/// The SDP for `monitor_id`'s video, with `track_url` as the stream's
/// control URL.
pub fn describe(monitor_id: u32, codec: VideoCodec, keyframe: &[u8], track_url: &str) -> String {
    let sets = parameter_sets(codec, keyframe);
    let b64 = |s: Option<&[u8]>| s.map(|s| STANDARD.encode(s));
    let (rtpmap, fmtp) = match codec {
        VideoCodec::H265 => {
            let mut fmtp = Vec::new();
            for (name, set) in [("vps", sets.vps), ("sps", sets.sps), ("pps", sets.pps)] {
                if let Some(v) = b64(set) {
                    fmtp.push(format!("sprop-{name}={v}"));
                }
            }
            ("H265/90000", fmtp.join(";"))
        }
        VideoCodec::H264 | VideoCodec::Unknown => {
            let mut fmtp = vec!["packetization-mode=1".to_string()];
            if let Some(sps) = sets.sps.filter(|s| s.len() >= 4) {
                fmtp.push(format!(
                    "profile-level-id={:02X}{:02X}{:02X}",
                    sps[1], sps[2], sps[3]
                ));
            }
            if let (Some(sps), Some(pps)) = (b64(sets.sps), b64(sets.pps)) {
                fmtp.push(format!("sprop-parameter-sets={sps},{pps}"));
            }
            ("H264/90000", fmtp.join(";"))
        }
    };
    let pt = PAYLOAD_TYPE;
    let mut sdp = format!(
        "v=0\r\n\
         o=- {monitor_id} 1 IN IP4 0.0.0.0\r\n\
         s=ZoneMinder monitor {monitor_id}\r\n\
         t=0 0\r\n\
         a=tool:zm-api\r\n\
         a=control:*\r\n\
         a=range:npt=now-\r\n\
         m=video 0 RTP/AVP {pt}\r\n\
         c=IN IP4 0.0.0.0\r\n\
         a=rtpmap:{pt} {rtpmap}\r\n"
    );
    if !fmtp.is_empty() {
        sdp.push_str(&format!("a=fmtp:{pt} {fmtp}\r\n"));
    }
    sdp.push_str(&format!("a=control:{track_url}\r\n"));
    sdp
}

#[cfg(test)]
mod tests {
    use super::*;

    const H264: &[u8] = include_bytes!("testdata/h264.annexb");

    #[test]
    fn h264_sdp_carries_profile_and_parameter_sets() {
        let sdp = describe(3, VideoCodec::H264, H264, "rtsp://h/live/3/track0?st=x");
        let sets = parameter_sets(VideoCodec::H264, H264);
        let sps = sets.sps.unwrap();
        assert!(sdp.contains("a=rtpmap:96 H264/90000\r\n"));
        assert!(sdp.contains(&format!(
            "profile-level-id={:02X}{:02X}{:02X}",
            sps[1], sps[2], sps[3]
        )));
        assert!(sdp.contains(&format!(
            "sprop-parameter-sets={},{}",
            STANDARD.encode(sps),
            STANDARD.encode(sets.pps.unwrap())
        )));
        assert!(sdp.ends_with("a=control:rtsp://h/live/3/track0?st=x\r\n"));
    }

    #[test]
    fn h265_sdp_lists_vps_sps_pps() {
        let h265 = include_bytes!("testdata/h265.annexb");
        let sdp = describe(1, VideoCodec::H265, h265, "u");
        assert!(sdp.contains("a=rtpmap:96 H265/90000\r\n"));
        for name in ["sprop-vps=", "sprop-sps=", "sprop-pps="] {
            assert!(sdp.contains(name), "{name} in {sdp}");
        }
    }

    #[test]
    fn missing_parameter_sets_leave_them_out() {
        let sdp = describe(1, VideoCodec::H264, &[0, 0, 1, 0x65, 1], "u");
        assert!(sdp.contains("a=fmtp:96 packetization-mode=1\r\n"));
    }
}
