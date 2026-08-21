use crate::entity::monitors;
use fake::Dummy;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Dummy)]
pub struct MonitorResponse {
    pub id: u32,
    pub name: String,
    pub deleted: bool,
    /// Whether the monitor is enabled (ZoneMinder's `Monitors.Enabled`).
    pub enabled: u8,
    pub notes: Option<String>,
    pub server_id: Option<u32>,
    pub storage_id: u16,
    pub manufacturer_id: Option<u32>,
    pub model_id: Option<u32>,
    pub r#type: String,
    pub function: String,
    pub capturing: String,
    pub decoding_enabled: u8,
    pub decoding: String,
    pub rtsp2_web_enabled: i8,
    pub rtsp2_web_type: String,
    pub janus_enabled: i8,
    pub janus_audio_enabled: i8,
    pub janus_profile_override: Option<String>,
    pub restream: i8,
    pub rtsp_user: Option<i32>,
    pub janus_rtsp_session_timeout: Option<i32>,
    pub linked_monitors: Option<String>,
    pub triggers: String,
    pub event_start_command: String,
    pub event_end_command: String,
    pub onvif_url: String,
    pub onvif_events_path: String,
    pub onvif_username: String,
    // The camera's ONVIF password is deliberately absent: read access to a
    // monitor must not hand out credentials for the camera itself.
    pub onvif_options: String,
    pub onvif_event_listener: i8,
    pub onvif_alarm_text: Option<String>,
    pub use_amcrest_api: i8,
    pub device: String,
    pub channel: u8,
    pub format: u32,
    pub v4l_multi_buffer: Option<u8>,
    pub v4l_captures_per_frame: Option<u8>,
    pub protocol: Option<String>,
    pub method: Option<String>,
    pub host: Option<String>,
    pub port: String,
    pub sub_path: String,
    pub path: Option<String>,
    pub second_path: Option<String>,
    pub options: Option<String>,
    pub user: Option<String>,
    // The camera's RTSP password (`pass`) is deliberately absent — see
    // `onvif_password` above.
    pub width: u16,
    pub height: u16,
    pub colours: u8,
    pub palette: u32,
    pub orientation: String,
    pub deinterlacing: u32,
    pub decoder: Option<String>,
    pub decoder_hw_accel_name: Option<String>,
    pub decoder_hw_accel_device: Option<String>,
    pub encoder_hw_accel_name: Option<String>,
    pub encoder_hw_accel_device: Option<String>,
    pub wall_clock_timestamps: i8,
    pub default_player: Option<String>,
    pub go2rtc_enabled: i8,
    /// Stream source channel. Replaces the old `RTSP2WebStream` column, which
    /// ZoneMinder renamed to `StreamChannel` in 1.37.79.
    pub stream_channel: String,
    pub save_jpe_gs: i8,
    pub video_writer: i8,
    pub output_codec: Option<u32>,
    pub encoder: Option<String>,
    pub output_container: Option<String>,
    pub encoder_parameters: Option<String>,
    pub record_audio: i8,
    pub recording_source: String,
    pub rtsp_describe: Option<u8>,
    pub brightness: Option<i32>,
    pub contrast: Option<i32>,
    pub hue: Option<i32>,
    pub colour: Option<i32>,
    pub event_prefix: String,
    pub label_format: Option<String>,
    pub label_x: u16,
    pub label_y: u16,
    pub label_size: u16,
    pub image_buffer_count: u16,
    pub max_image_buffer_count: u16,
    pub warmup_count: u16,
    pub pre_event_count: u16,
    pub post_event_count: u16,
    pub stream_replay_buffer: u32,
    pub alarm_frame_count: u16,
    pub section_length: u32,
    pub section_length_warn: i8,
    pub event_close_mode: String,
    pub min_section_length: u32,
    pub frame_skip: u16,
    pub motion_frame_skip: u16,
    pub analysis_fps_limit: Option<f64>,
    pub analysis_update_delay: u16,
    pub max_fps: Option<f64>,
    pub alarm_max_fps: Option<f64>,
    pub fps_report_interval: u16,
    pub ref_blend_perc: u8,
    pub alarm_ref_blend_perc: u8,
    pub controllable: u8,
    pub control_id: Option<u32>,
    pub control_device: Option<String>,
    pub control_address: Option<String>,
    pub auto_stop_timeout: Option<f64>,
    pub track_motion: u8,
    pub track_delay: Option<u16>,
    pub return_location: i8,
    pub return_delay: Option<u16>,
    pub modect_during_ptz: u8,
    pub default_rate: u16,
    pub default_scale: String,
    pub default_codec: String,
    pub signal_check_points: u32,
    pub signal_check_colour: String,
    pub web_colour: String,
    pub exif: u8,
    pub sequence: Option<u16>,
    pub zone_count: i8,
    pub refresh: Option<u32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub rtsp_server: i8,
    pub rtsp_stream_name: String,
    pub soap_wsa_compl: i8,
    pub importance: String,
    pub mqtt_enabled: i8,
    pub mqtt_subscriptions: String,
    pub startup_delay: i32,
    pub analysing: String,
    pub analysis_source: String,
    pub analysis_image: String,
    pub recording: String,
}

/// Serialize a ZoneMinder DB enum to the variant-name form the request DTOs
/// accept (e.g. `Rotate90`, `System`, `Auto`), not the SCREAMING DB value that
/// `.to_string()` yields (`ROTATE_90`, `system`). Keeps GET → POST round trips
/// (clone / import) from 422ing on the casing mismatch (GH #18).
fn enum_str<T: serde::Serialize>(v: &T) -> String {
    match serde_json::to_value(v) {
        Ok(serde_json::Value::String(s)) => s,
        other => other.map(|o| o.to_string()).unwrap_or_default(),
    }
}

impl From<monitors::Model> for MonitorResponse {
    fn from(model: monitors::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            deleted: model.deleted != 0,
            enabled: model.enabled,
            notes: model.notes,
            server_id: model.server_id,
            storage_id: model.storage_id.unwrap_or(0),
            manufacturer_id: model.manufacturer_id,
            model_id: model.model_id,
            r#type: enum_str(&model.r#type),
            function: enum_str(&model.function),
            capturing: enum_str(&model.capturing),
            decoding_enabled: model.decoding_enabled,
            decoding: enum_str(&model.decoding),
            rtsp2_web_enabled: model.rtsp2_web_enabled,
            rtsp2_web_type: enum_str(&model.rtsp2_web_type),
            janus_enabled: model.janus_enabled,
            janus_audio_enabled: model.janus_audio_enabled,
            janus_profile_override: Some(model.janus_profile_override),
            restream: model.restream,
            rtsp_user: model.rtsp_user,
            janus_rtsp_session_timeout: Some(model.janus_rtsp_session_timeout),
            linked_monitors: model.linked_monitors,
            triggers: model.triggers,
            event_start_command: model.event_start_command,
            event_end_command: model.event_end_command,
            onvif_url: model.onvif_url,
            onvif_events_path: model.onvif_events_path,
            onvif_username: model.onvif_username,
            onvif_options: model.onvif_options,
            onvif_event_listener: model.onvif_event_listener,
            onvif_alarm_text: model.onvif_alarm_text,
            use_amcrest_api: model.use_amcrest_api,
            device: model.device,
            channel: model.channel,
            format: model.format,
            v4l_multi_buffer: model.v4l_multi_buffer,
            v4l_captures_per_frame: model.v4l_captures_per_frame,
            protocol: model.protocol,
            method: model.method,
            host: model.host,
            port: model.port,
            sub_path: model.sub_path,
            path: model.path,
            second_path: model.second_path,
            options: model.options,
            user: model.user,
            width: model.width,
            height: model.height,
            colours: model.colours,
            palette: model.palette,
            orientation: enum_str(&model.orientation),
            deinterlacing: model.deinterlacing,
            decoder: model.decoder,
            decoder_hw_accel_name: model.decoder_hw_accel_name,
            decoder_hw_accel_device: model.decoder_hw_accel_device,
            encoder_hw_accel_name: model.encoder_hw_accel_name,
            encoder_hw_accel_device: model.encoder_hw_accel_device,
            wall_clock_timestamps: model.wall_clock_timestamps,
            default_player: model.default_player,
            go2rtc_enabled: model.go2_rtc_enabled,
            stream_channel: enum_str(&model.stream_channel),
            save_jpe_gs: model.save_jpe_gs,
            video_writer: model.video_writer,
            output_codec: Some(model.output_codec),
            encoder: model.encoder,
            output_container: model.output_container.as_ref().map(enum_str),
            encoder_parameters: model.encoder_parameters,
            record_audio: model.record_audio,
            recording_source: enum_str(&model.recording_source),
            rtsp_describe: model.rtsp_describe,
            brightness: model.brightness,
            contrast: model.contrast,
            hue: model.hue,
            colour: model.colour,
            event_prefix: model.event_prefix,
            label_format: model.label_format,
            label_x: model.label_x,
            label_y: model.label_y,
            label_size: model.label_size,
            image_buffer_count: model.image_buffer_count,
            max_image_buffer_count: model.max_image_buffer_count,
            warmup_count: model.warmup_count,
            pre_event_count: model.pre_event_count,
            post_event_count: model.post_event_count,
            stream_replay_buffer: model.stream_replay_buffer,
            alarm_frame_count: model.alarm_frame_count,
            section_length: model.section_length,
            section_length_warn: model.section_length_warn,
            event_close_mode: enum_str(&model.event_close_mode),
            min_section_length: model.min_section_length,
            frame_skip: model.frame_skip,
            motion_frame_skip: model.motion_frame_skip,
            analysis_fps_limit: model.analysis_fps_limit.map(|d| d.to_f64().unwrap_or(0.0)),
            analysis_update_delay: model.analysis_update_delay,
            max_fps: model.max_fps.map(|d| d.to_f64().unwrap_or(0.0)),
            alarm_max_fps: model.alarm_max_fps.map(|d| d.to_f64().unwrap_or(0.0)),
            fps_report_interval: model.fps_report_interval,
            ref_blend_perc: model.ref_blend_perc,
            alarm_ref_blend_perc: model.alarm_ref_blend_perc,
            controllable: model.controllable,
            control_id: model.control_id,
            control_device: model.control_device,
            control_address: model.control_address,
            auto_stop_timeout: model.auto_stop_timeout.map(|d| d.to_f64().unwrap_or(0.0)),
            track_motion: model.track_motion,
            track_delay: model.track_delay,
            return_location: model.return_location,
            return_delay: model.return_delay,
            modect_during_ptz: model.modect_during_ptz,
            default_rate: model.default_rate,
            default_scale: model.default_scale.to_string(),
            default_codec: enum_str(&model.default_codec),
            signal_check_points: model.signal_check_points,
            signal_check_colour: model.signal_check_colour,
            web_colour: model.web_colour,
            exif: model.exif,
            sequence: model.sequence,
            zone_count: model.zone_count,
            refresh: model.refresh,
            latitude: model.latitude.map(|d| d.to_f64().unwrap_or(0.0)),
            longitude: model.longitude.map(|d| d.to_f64().unwrap_or(0.0)),
            rtsp_server: model.rtsp_server,
            rtsp_stream_name: model.rtsp_stream_name,
            soap_wsa_compl: model.soap_wsa_compl,
            importance: enum_str(&model.importance),
            mqtt_enabled: model.mqtt_enabled,
            mqtt_subscriptions: model.mqtt_subscriptions.unwrap_or_default(),
            startup_delay: model.startup_delay,
            analysing: enum_str(&model.analysing),
            analysis_source: enum_str(&model.analysis_source),
            analysis_image: enum_str(&model.analysis_image),
            recording: enum_str(&model.recording),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Fake, Faker};

    /// Read access to a monitor must never expose the camera's own
    /// credentials: the serialized response has no password-bearing keys.
    #[test]
    fn monitor_response_never_serializes_camera_credentials() {
        let resp: MonitorResponse = Faker.fake();
        let json = serde_json::to_value(&resp).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("pass"));
        assert!(!obj.contains_key("onvif_password"));
    }

    /// GH #18: `enum_str` emits the request-accepted variant name (not the
    /// SCREAMING DB value), and that string deserializes straight back into the
    /// request enum — so a GET → POST round trip does not 422 on casing.
    #[test]
    fn enum_str_round_trips_into_request_enums() {
        use crate::entity::sea_orm_active_enums::{
            DefaultCodec, EventCloseMode, Orientation, Rtsp2WebType,
        };

        assert_eq!(enum_str(&Orientation::Rotate90), "Rotate90");
        assert_eq!(enum_str(&EventCloseMode::System), "System");
        assert_eq!(enum_str(&DefaultCodec::Auto), "Auto");
        assert_eq!(enum_str(&Rtsp2WebType::WebRtc), "WebRtc");

        // The emitted string parses back into the same request enum.
        let s = enum_str(&Orientation::Rotate90);
        let back: Orientation = serde_json::from_value(serde_json::Value::String(s)).unwrap();
        assert_eq!(back, Orientation::Rotate90);
    }
}
