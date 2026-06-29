// src/dto/request/monitor.rs
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
// use sea_orm::ActiveEnum; // Unused import
use crate::entity::sea_orm_active_enums::{
    Analysing, AnalysisImage, AnalysisSource, Capturing, Decoding, DefaultCodec, EventCloseMode,
    Function, Importance, MonitorType, Orientation, OutputContainer, Recording, RecordingSource,
    Rtsp2WebType,
};

// Custom validator for state actions
pub fn is_valid_state(value: &str, _ctx: &()) -> garde::Result {
    match value {
        "start" | "stop" | "restart" => Ok(()),
        _ => Err(garde::Error::new(
            "invalid state; must be 'start', 'stop', or 'restart'",
        )),
    }
}

// Custom validator for alarm actions
pub fn is_valid_alarm_action(value: &str, _ctx: &()) -> garde::Result {
    match value {
        "on" | "off" | "status" => Ok(()),
        _ => Err(garde::Error::new(
            "invalid alarm action; must be 'on', 'off', or 'status'",
        )),
    }
}

// Hardening for fields that flow into URLs, hostnames, file paths, or process
// argv assembled by ZoneMinder's capture daemon. We don't try to validate URL
// syntax — that's the camera library's job — but we reject inputs that have no
// business in any URL component:
//
//   * NUL, control characters, and CR/LF: header smuggling, log injection, and
//     C-string truncation downstream.
//   * Lengths beyond what any real camera URL would need: bounded memory cost
//     and a sanity cap when these values get serialised back into config or
//     argv.
//
// Used on `host`, `path`, `second_path`, `port`, and similar URL components.
pub fn is_safe_url_component(value: &str, _ctx: &()) -> garde::Result {
    if value.len() > 2048 {
        return Err(garde::Error::new("must be at most 2048 characters"));
    }
    if value.bytes().any(|b| b < 0x20 || b == 0x7F) {
        return Err(garde::Error::new(
            "must not contain NUL, control characters, or CR/LF",
        ));
    }
    Ok(())
}

// Stricter rules for fields that take a full URL (currently only `onvif_url`).
// In addition to the component-level checks, restrict the scheme to the small
// set that makes sense for a network camera, blocking SSRF gadgets like
// `file://`, `unix://`, `gopher://`, and `javascript:`.
pub fn is_safe_onvif_url(value: &str, ctx: &()) -> garde::Result {
    is_safe_url_component(value, ctx)?;
    if let Some(idx) = value.find("://") {
        let scheme = value[..idx].to_ascii_lowercase();
        if !matches!(scheme.as_str(), "http" | "https" | "rtsp" | "rtsps") {
            return Err(garde::Error::new(
                "unsupported URL scheme; expected http, https, rtsp, or rtsps",
            ));
        }
    }
    Ok(())
}

// `event_start_command` and `event_end_command` are by design shell commands
// that ZoneMinder execs, so we cannot block shell metacharacters without
// breaking the feature. We can still bound the field and reject NUL and
// newlines, which would otherwise enable C-string truncation and log
// forgery in the daemon that runs the command.
pub fn is_safe_command_string(value: &str, _ctx: &()) -> garde::Result {
    if value.len() > 4096 {
        return Err(garde::Error::new("must be at most 4096 characters"));
    }
    if value.bytes().any(|b| b == 0 || b == b'\n' || b == b'\r') {
        return Err(garde::Error::new(
            "must not contain NUL or newline characters",
        ));
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateMonitorRequest {
    #[garde(length(min = 1, max = 64))]
    pub name: String,
    #[garde(skip)] // Boolean type doesn't need validation
    pub deleted: bool,
    #[garde(skip)] // Option<String> can be None, or we could validate if Some
    pub notes: Option<String>,
    #[garde(skip)] // Option<u32> can be None, or we could validate if Some
    pub server_id: Option<u32>,
    #[garde(range(min = 1))]
    pub storage_id: u16,
    #[garde(skip)] // Option<u32> can be None
    pub manufacturer_id: Option<u32>,
    #[garde(skip)] // Option<u32> can be None
    pub model_id: Option<u32>,

    // Enum field validations
    #[serde(rename = "type")]
    #[garde(skip)]
    pub r#type: MonitorType,

    #[serde(rename = "function")]
    #[garde(skip)]
    pub function: Function,

    #[serde(rename = "capturing")]
    #[garde(skip)]
    pub capturing: Capturing,

    #[garde(range(min = 0, max = 1))]
    pub decoding_enabled: u8,

    #[serde(rename = "decoding")]
    #[garde(skip)]
    pub decoding: Decoding,

    #[garde(range(min = -1, max = 1))]
    pub rtsp2_web_enabled: i8,

    #[serde(rename = "rtsp2_web_type")]
    #[garde(skip)]
    pub rtsp2_web_type: Rtsp2WebType,

    #[garde(range(min = -1, max = 1))]
    pub janus_enabled: i8,

    #[garde(range(min = -1, max = 1))]
    pub janus_audio_enabled: i8,

    #[garde(skip)] // Option<String> can be None
    pub janus_profile_override: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub restream: i8,

    #[garde(skip)] // Option<i32> can be None
    pub rtsp_user: Option<i32>,

    #[garde(skip)] // Option<i32> can be None
    pub janus_rtsp_session_timeout: Option<i32>,

    #[garde(skip)] // Option<String> can be None
    pub linked_monitors: Option<String>,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub triggers: String,

    // DB column `EventStartCommand` is varchar(255).
    #[garde(length(max = 255))]
    #[garde(custom(is_safe_command_string))]
    pub event_start_command: String,

    // DB column `EventEndCommand` is varchar(255).
    #[garde(length(max = 255))]
    #[garde(custom(is_safe_command_string))]
    pub event_end_command: String,

    // DB column `ONVIF_URL` is varchar(255).
    #[garde(length(max = 255))]
    #[garde(custom(is_safe_onvif_url))]
    pub onvif_url: String,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub onvif_events_path: String,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub onvif_username: String,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub onvif_password: String,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub onvif_options: String,

    #[garde(range(min = -1, max = 1))]
    pub onvif_event_listener: i8,

    #[garde(skip)] // Option<String> can be None
    pub onvif_alarm_text: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub use_amcrest_api: i8,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub device: String,

    #[garde(range(min = 0))]
    pub channel: u8,

    #[garde(skip)] // u32 doesn't need range validation in this case
    pub format: u32,

    #[garde(skip)] // Option<u8> can be None
    pub v4l_multi_buffer: Option<u8>,

    #[garde(skip)] // Option<u8> can be None
    pub v4l_captures_per_frame: Option<u8>,

    #[garde(skip)] // Option<String> can be None
    pub protocol: Option<String>,

    #[garde(skip)] // Option<String> can be None
    pub method: Option<String>,

    // DB column `Host` is varchar(64).
    #[garde(inner(length(max = 64)))]
    #[garde(inner(custom(is_safe_url_component)))]
    pub host: Option<String>,

    // DB column `Port` is varchar(8).
    #[garde(length(max = 8))]
    #[garde(custom(is_safe_url_component))]
    pub port: String,

    // DB column `SubPath` is varchar(64).
    #[garde(length(max = 64))]
    #[garde(custom(is_safe_url_component))]
    pub sub_path: String,

    // DB column `Path` is varchar(255).
    #[garde(inner(length(max = 255)))]
    #[garde(inner(custom(is_safe_url_component)))]
    pub path: Option<String>,

    // DB column `SecondPath` is varchar(255).
    #[garde(inner(length(max = 255)))]
    #[garde(inner(custom(is_safe_url_component)))]
    pub second_path: Option<String>,

    #[garde(skip)] // Option<String> can be None
    pub options: Option<String>,

    #[garde(skip)] // Option<String> can be None
    pub user: Option<String>,

    #[garde(skip)] // Option<String> can be None
    pub pass: Option<String>,

    #[garde(range(min = 1))]
    pub width: u16,

    #[garde(range(min = 1))]
    pub height: u16,

    #[garde(range(min = 0))]
    pub colours: u8,

    #[garde(skip)] // u32 doesn't need validation in this case
    pub palette: u32,

    #[serde(rename = "orientation")]
    #[garde(skip)]
    pub orientation: Orientation,

    #[garde(skip)] // u32 doesn't need validation in this case
    pub deinterlacing: u32,

    #[garde(skip)] // Option<String> can be None
    pub decoder: Option<String>,

    #[garde(skip)] // Option<String> can be None
    pub decoder_hw_accel_name: Option<String>,

    #[garde(skip)] // Option<String> can be None
    pub decoder_hw_accel_device: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub save_jpe_gs: i8,

    #[garde(range(min = -1, max = 1))]
    pub video_writer: i8,

    #[garde(skip)] // Option<u32> can be None
    pub output_codec: Option<u32>,

    #[garde(skip)] // Option<String> can be None
    pub encoder: Option<String>,

    #[serde(rename = "output_container")]
    #[garde(skip)]
    pub output_container: OutputContainer,

    #[garde(skip)] // Option<String> can be None
    pub encoder_parameters: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub record_audio: i8,

    #[serde(rename = "recording_source")]
    #[garde(skip)]
    pub recording_source: RecordingSource,

    #[garde(skip)] // Option<u8> can be None
    pub rtsp_describe: Option<u8>,

    #[garde(range(min = 0, max = i32::MAX))]
    pub brightness: i32,

    #[garde(range(min = 0, max = i32::MAX))] // Validate range
    pub contrast: i32,

    #[garde(range(min = 0, max = i32::MAX))]
    pub hue: i32,

    #[garde(range(min = 0, max = i32::MAX))]
    pub colour: i32,

    #[garde(length(min = 0))] // Empty string is valid
    pub event_prefix: String,

    #[garde(skip)] // Option<String> can be None
    pub label_format: Option<String>,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub label_x: u16,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub label_y: u16,

    #[garde(range(min = 1))]
    pub label_size: u16,

    #[garde(range(min = 1))]
    pub image_buffer_count: u16,

    #[garde(range(min = 1))]
    pub max_image_buffer_count: u16,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub warmup_count: u16,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub pre_event_count: u16,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub post_event_count: u16,

    #[garde(range(min = 1))]
    pub stream_replay_buffer: u32,

    #[garde(range(min = 1))]
    pub alarm_frame_count: u16,

    #[garde(range(min = 1))]
    pub section_length: u32,

    #[garde(range(min = -1, max = 1))]
    pub section_length_warn: i8,

    #[serde(rename = "event_close_mode")]
    #[garde(skip)]
    pub event_close_mode: EventCloseMode,

    #[garde(range(min = 1))]
    pub min_section_length: u32,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub frame_skip: u16,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub motion_frame_skip: u16,

    #[garde(skip)] // Option<f64> can be None
    pub analysis_fps_limit: Option<f64>,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub analysis_update_delay: u16,

    #[garde(skip)] // Option<f64> can be None
    pub max_fps: Option<f64>,

    #[garde(skip)] // Option<f64> can be None
    pub alarm_max_fps: Option<f64>,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub fps_report_interval: u16,

    #[garde(range(min = 0, max = 100))]
    pub ref_blend_perc: u8,

    #[garde(range(min = 0, max = 100))]
    pub alarm_ref_blend_perc: u8,

    #[garde(range(min = 0, max = 1))]
    pub controllable: u8,

    #[garde(skip)] // Option<u32> can be None
    pub control_id: Option<u32>,

    #[garde(skip)] // Option<String> can be None
    pub control_device: Option<String>,

    #[garde(skip)] // Option<String> can be None
    pub control_address: Option<String>,

    #[garde(skip)] // Option<f64> can be None
    pub auto_stop_timeout: Option<f64>,

    #[garde(range(min = 0, max = 1))]
    pub track_motion: u8,

    #[garde(skip)] // Option<u16> can be None
    pub track_delay: Option<u16>,

    #[garde(range(min = -1, max = 1))]
    pub return_location: i8,

    #[garde(skip)] // Option<u16> can be None
    pub return_delay: Option<u16>,

    #[garde(range(min = 0, max = 1))]
    pub modect_during_ptz: u8,

    #[garde(skip)]
    pub default_rate: u16,

    #[garde(length(min = 0))]
    pub default_scale: String,

    #[serde(rename = "default_codec")]
    #[garde(skip)]
    pub default_codec: DefaultCodec,

    #[garde(skip)] // u32 doesn't need validation in this case
    pub signal_check_points: u32,

    #[garde(length(min = 0))] // Empty string is valid
    pub signal_check_colour: String,

    #[garde(length(min = 0))] // Empty string is valid
    pub web_colour: String,

    #[garde(range(min = 0, max = 1))]
    pub exif: u8,

    #[garde(skip)] // Option<u16> can be None
    pub sequence: Option<u16>,

    #[garde(range(min = -128, max = 127))]
    pub zone_count: i8,

    #[garde(skip)] // Option<u32> can be None
    pub refresh: Option<u32>,

    #[garde(skip)] // Option<f64> can be None
    pub latitude: Option<f64>,

    #[garde(skip)] // Option<f64> can be None
    pub longitude: Option<f64>,

    #[garde(range(min = -1, max = 1))]
    pub rtsp_server: i8,

    #[garde(length(min = 0))] // Empty string is valid
    pub rtsp_stream_name: String,

    #[garde(range(min = -1, max = 1))]
    pub soap_wsa_compl: i8,

    // These are already enum types, so they're validated by the type system
    #[serde(rename = "importance")]
    #[garde(skip)]
    pub importance: Importance,

    #[garde(range(min = -1, max = 1))]
    pub mqtt_enabled: i8,

    #[garde(length(min = 0))] // Empty string is valid
    pub mqtt_subscriptions: String,

    #[garde(skip)] // i32 doesn't need validation in this case
    pub startup_delay: i32,

    #[serde(rename = "analysing")]
    #[garde(skip)]
    pub analysing: Analysing,

    #[garde(skip)]
    pub analysis_source: AnalysisSource,

    #[garde(skip)]
    pub analysis_image: AnalysisImage,

    #[garde(skip)]
    pub recording: Recording,
}

impl Default for CreateMonitorRequest {
    /// ZoneMinder-sensible, **validation-passing** defaults for every field, so
    /// callers (e.g. ONVIF onboarding) can build a valid monitor by overriding
    /// only the handful they care about: `name`, `type`, `path`, `width`,
    /// `height`, `storage_id`, and the `onvif_*` fields. The
    /// `create_monitor_request_default_passes_validation` test guards that this
    /// satisfies every `garde` constraint.
    fn default() -> Self {
        Self {
            // Placeholder so the Default itself validates (min length 1); real
            // callers (e.g. onboarding) always override this.
            name: "Monitor".to_string(),
            deleted: false,
            notes: None,
            server_id: None,
            storage_id: 1,
            manufacturer_id: None,
            model_id: None,
            r#type: MonitorType::Ffmpeg,
            function: Function::Monitor,
            capturing: Capturing::Always,
            decoding_enabled: 1,
            decoding: Decoding::Always,
            rtsp2_web_enabled: 0,
            rtsp2_web_type: Rtsp2WebType::Hls,
            janus_enabled: 0,
            janus_audio_enabled: 0,
            janus_profile_override: None,
            restream: 0,
            rtsp_user: None,
            janus_rtsp_session_timeout: None,
            linked_monitors: None,
            triggers: String::new(),
            event_start_command: String::new(),
            event_end_command: String::new(),
            onvif_url: String::new(),
            onvif_events_path: String::new(),
            onvif_username: String::new(),
            onvif_password: String::new(),
            onvif_options: String::new(),
            onvif_event_listener: 0,
            onvif_alarm_text: None,
            use_amcrest_api: 0,
            device: String::new(),
            channel: 0,
            format: 0,
            v4l_multi_buffer: None,
            v4l_captures_per_frame: None,
            protocol: None,
            method: None,
            host: None,
            port: String::new(),
            sub_path: String::new(),
            path: None,
            second_path: None,
            options: None,
            user: None,
            pass: None,
            width: 1,
            height: 1,
            colours: 4,
            palette: 0,
            orientation: Orientation::Rotate0,
            deinterlacing: 0,
            decoder: None,
            decoder_hw_accel_name: None,
            decoder_hw_accel_device: None,
            save_jpe_gs: 1,
            video_writer: 1,
            output_codec: None,
            encoder: None,
            output_container: OutputContainer::Auto,
            encoder_parameters: None,
            record_audio: 1,
            recording_source: RecordingSource::Primary,
            rtsp_describe: None,
            brightness: 0,
            contrast: 0,
            hue: 0,
            colour: 0,
            event_prefix: "Event-".to_string(),
            label_format: None,
            label_x: 0,
            label_y: 0,
            label_size: 1,
            image_buffer_count: 3,
            max_image_buffer_count: 3,
            warmup_count: 0,
            pre_event_count: 5,
            post_event_count: 5,
            stream_replay_buffer: 1000,
            alarm_frame_count: 1,
            section_length: 600,
            section_length_warn: 0,
            event_close_mode: EventCloseMode::Idle,
            min_section_length: 10,
            frame_skip: 0,
            motion_frame_skip: 0,
            analysis_fps_limit: None,
            analysis_update_delay: 0,
            max_fps: None,
            alarm_max_fps: None,
            fps_report_interval: 100,
            ref_blend_perc: 6,
            alarm_ref_blend_perc: 6,
            controllable: 0,
            control_id: None,
            control_device: None,
            control_address: None,
            auto_stop_timeout: None,
            track_motion: 0,
            track_delay: None,
            return_location: -1,
            return_delay: None,
            modect_during_ptz: 0,
            default_rate: 100,
            default_scale: "0".to_string(),
            default_codec: DefaultCodec::Auto,
            signal_check_points: 0,
            signal_check_colour: String::new(),
            web_colour: String::new(),
            exif: 0,
            sequence: None,
            zone_count: 0,
            refresh: None,
            latitude: None,
            longitude: None,
            rtsp_server: 0,
            rtsp_stream_name: String::new(),
            soap_wsa_compl: 0,
            importance: Importance::Normal,
            mqtt_enabled: 0,
            mqtt_subscriptions: String::new(),
            startup_delay: 0,
            analysing: Analysing::None,
            analysis_source: AnalysisSource::Primary,
            analysis_image: AnalysisImage::FullColour,
            recording: Recording::Always,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateMonitorRequest {
    #[garde(length(min = 1, max = 64))]
    pub name: Option<String>,

    #[garde(range(min = 0, max = 1))]
    pub deleted: Option<i8>,

    #[garde(skip)]
    pub notes: Option<String>,

    #[garde(skip)]
    pub server_id: Option<u32>,

    #[garde(range(min = 1))]
    pub storage_id: Option<u16>,

    #[garde(skip)]
    pub manufacturer_id: Option<u32>,

    #[garde(skip)]
    pub model_id: Option<u32>,

    #[serde(rename = "type")]
    #[garde(skip)]
    pub r#type: Option<MonitorType>,

    #[serde(rename = "function")]
    #[garde(skip)]
    pub function: Option<Function>,

    #[serde(rename = "capturing")]
    #[garde(skip)]
    pub capturing: Option<Capturing>,

    #[garde(range(min = 0, max = 1))]
    pub decoding_enabled: Option<u8>,

    #[serde(rename = "decoding")]
    #[garde(skip)]
    pub decoding: Option<Decoding>,

    #[garde(range(min = -1, max = 1))]
    pub rtsp2_web_enabled: Option<i8>,

    #[serde(rename = "rtsp2_web_type")]
    #[garde(skip)]
    pub rtsp2_web_type: Option<Rtsp2WebType>,

    #[garde(range(min = -1, max = 1))]
    pub janus_enabled: Option<i8>,

    #[garde(range(min = -1, max = 1))]
    pub janus_audio_enabled: Option<i8>,

    #[garde(skip)]
    pub janus_profile_override: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub restream: Option<i8>,

    #[garde(skip)]
    pub rtsp_user: Option<i32>,

    #[garde(skip)]
    pub janus_rtsp_session_timeout: Option<i32>,

    #[garde(skip)]
    pub linked_monitors: Option<String>,

    #[garde(length(min = 0))]
    pub triggers: Option<String>,

    #[garde(inner(length(max = 255)))]
    #[garde(inner(custom(is_safe_command_string)))]
    pub event_start_command: Option<String>,

    #[garde(inner(length(max = 255)))]
    #[garde(inner(custom(is_safe_command_string)))]
    pub event_end_command: Option<String>,

    #[garde(inner(length(max = 255)))]
    #[garde(inner(custom(is_safe_onvif_url)))]
    pub onvif_url: Option<String>,

    #[garde(length(min = 0))]
    pub onvif_events_path: Option<String>,

    #[garde(length(min = 0))]
    pub onvif_username: Option<String>,

    #[garde(length(min = 0))]
    pub onvif_password: Option<String>,

    #[garde(length(min = 0))]
    pub onvif_options: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub onvif_event_listener: Option<i8>,

    #[garde(skip)]
    pub onvif_alarm_text: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub use_amcrest_api: Option<i8>,

    #[garde(length(min = 0))]
    pub device: Option<String>,

    #[garde(range(min = 0))]
    pub channel: Option<u8>,

    #[garde(skip)]
    pub format: Option<u32>,

    #[garde(skip)]
    pub v4l_multi_buffer: Option<u8>,

    #[garde(skip)]
    pub v4l_captures_per_frame: Option<u8>,

    #[garde(skip)]
    pub protocol: Option<String>,

    #[garde(skip)]
    pub method: Option<String>,

    #[garde(inner(length(max = 64)))]
    #[garde(inner(custom(is_safe_url_component)))]
    pub host: Option<String>,

    #[garde(inner(length(max = 8)))]
    #[garde(inner(custom(is_safe_url_component)))]
    pub port: Option<String>,

    #[garde(inner(length(max = 64)))]
    #[garde(inner(custom(is_safe_url_component)))]
    pub sub_path: Option<String>,

    #[garde(inner(length(max = 255)))]
    #[garde(inner(custom(is_safe_url_component)))]
    pub path: Option<String>,

    #[garde(inner(length(max = 255)))]
    #[garde(inner(custom(is_safe_url_component)))]
    pub second_path: Option<String>,

    #[garde(skip)]
    pub options: Option<String>,

    #[garde(skip)]
    pub user: Option<String>,

    #[garde(skip)]
    pub pass: Option<String>,

    #[garde(range(min = 1))]
    pub width: Option<u16>,

    #[garde(range(min = 1))]
    pub height: Option<u16>,

    #[garde(range(min = 0))]
    pub colours: Option<u8>,

    #[garde(skip)]
    pub palette: Option<u32>,

    #[serde(rename = "orientation")]
    #[garde(skip)]
    pub orientation: Option<Orientation>,

    #[garde(skip)]
    pub deinterlacing: Option<u32>,

    #[garde(skip)]
    pub decoder: Option<String>,

    #[garde(skip)]
    pub decoder_hw_accel_name: Option<String>,

    #[garde(skip)]
    pub decoder_hw_accel_device: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub save_jpe_gs: Option<i8>,

    #[garde(range(min = -1, max = 1))]
    pub video_writer: Option<i8>,

    #[garde(skip)]
    pub output_codec: Option<u32>,

    #[garde(skip)]
    pub encoder: Option<String>,

    #[serde(rename = "output_container")]
    #[garde(skip)]
    pub output_container: Option<OutputContainer>,

    #[garde(skip)]
    pub encoder_parameters: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub record_audio: Option<i8>,

    #[serde(rename = "recording_source")]
    #[garde(skip)]
    pub recording_source: Option<RecordingSource>,

    #[garde(skip)]
    pub rtsp_describe: Option<u8>,

    #[garde(range(min = 0, max = i32::MAX))]
    pub brightness: Option<i32>,

    #[garde(range(min = 0, max = i32::MAX))]
    pub contrast: Option<i32>,

    #[garde(range(min = 0, max = i32::MAX))]
    pub hue: Option<i32>,

    #[garde(range(min = 0, max = i32::MAX))]
    pub colour: Option<i32>,

    #[garde(length(min = 0))]
    pub event_prefix: Option<String>,

    #[garde(skip)]
    pub label_format: Option<String>,

    #[garde(skip)]
    pub label_x: Option<u16>,

    #[garde(skip)]
    pub label_y: Option<u16>,

    #[garde(range(min = 1))]
    pub label_size: Option<u16>,

    #[garde(range(min = 1))]
    pub image_buffer_count: Option<u16>,

    #[garde(range(min = 1))]
    pub max_image_buffer_count: Option<u16>,

    #[garde(skip)]
    pub warmup_count: Option<u16>,

    #[garde(skip)]
    pub pre_event_count: Option<u16>,

    #[garde(skip)]
    pub post_event_count: Option<u16>,

    #[garde(range(min = 1))]
    pub stream_replay_buffer: Option<u32>,

    #[garde(range(min = 1))]
    pub alarm_frame_count: Option<u16>,

    #[garde(range(min = 1))]
    pub section_length: Option<u32>,

    #[garde(range(min = -1, max = 1))]
    pub section_length_warn: Option<i8>,

    #[serde(rename = "event_close_mode")]
    #[garde(skip)]
    pub event_close_mode: Option<EventCloseMode>,

    #[garde(range(min = 1))]
    pub min_section_length: Option<u32>,

    #[garde(skip)]
    pub frame_skip: Option<u16>,

    #[garde(skip)]
    pub motion_frame_skip: Option<u16>,

    #[garde(skip)]
    pub analysis_fps_limit: Option<f64>,

    #[garde(skip)]
    pub analysis_update_delay: Option<u16>,

    #[garde(skip)]
    pub max_fps: Option<f64>,

    #[garde(skip)]
    pub alarm_max_fps: Option<f64>,

    #[garde(skip)]
    pub fps_report_interval: Option<u16>,

    #[garde(range(min = 0, max = 100))]
    pub ref_blend_perc: Option<u8>,

    #[garde(range(min = 0, max = 100))]
    pub alarm_ref_blend_perc: Option<u8>,

    #[garde(range(min = 0, max = 1))]
    pub controllable: Option<u8>,

    #[garde(skip)]
    pub control_id: Option<u32>,

    #[garde(skip)]
    pub control_device: Option<String>,

    #[garde(skip)]
    pub control_address: Option<String>,

    #[garde(skip)]
    pub auto_stop_timeout: Option<f64>,

    #[garde(range(min = 0, max = 1))]
    pub track_motion: Option<u8>,

    #[garde(skip)]
    pub track_delay: Option<u16>,

    #[garde(range(min = -1, max = 1))]
    pub return_location: Option<i8>,

    #[garde(skip)]
    pub return_delay: Option<u16>,

    #[garde(range(min = 0, max = 1))]
    pub modect_during_ptz: Option<u8>,

    #[garde(skip)]
    pub default_rate: Option<u16>,

    #[garde(length(min = 0))]
    pub default_scale: Option<String>,

    #[serde(rename = "default_codec")]
    #[garde(skip)]
    pub default_codec: Option<DefaultCodec>,

    #[garde(skip)]
    pub signal_check_points: Option<u32>,

    #[garde(length(min = 0))]
    pub signal_check_colour: Option<String>,

    #[garde(length(min = 0))]
    pub web_colour: Option<String>,

    #[garde(range(min = 0, max = 1))]
    pub exif: Option<u8>,

    #[garde(skip)]
    pub sequence: Option<u16>,

    #[garde(range(min = -128, max = 127))]
    pub zone_count: Option<i8>,

    #[garde(skip)]
    pub refresh: Option<u32>,

    #[garde(skip)]
    pub latitude: Option<f64>,

    #[garde(skip)]
    pub longitude: Option<f64>,

    #[garde(range(min = -1, max = 1))]
    pub rtsp_server: Option<i8>,

    #[garde(length(min = 0))]
    pub rtsp_stream_name: Option<String>,

    #[garde(range(min = -1, max = 1))]
    pub soap_wsa_compl: Option<i8>,

    #[serde(rename = "importance")]
    #[garde(skip)]
    pub importance: Option<Importance>,

    #[garde(range(min = -1, max = 1))]
    pub mqtt_enabled: Option<i8>,

    #[garde(length(min = 0))]
    pub mqtt_subscriptions: Option<String>,

    #[garde(skip)]
    pub startup_delay: Option<i32>,

    #[serde(rename = "analysing")]
    #[garde(skip)]
    pub analysing: Option<Analysing>,

    #[serde(rename = "analysis_source")]
    #[garde(skip)]
    pub analysis_source: Option<AnalysisSource>,

    #[serde(rename = "analysis_image")]
    #[garde(skip)]
    pub analysis_image: Option<AnalysisImage>,

    #[serde(rename = "recording")]
    #[garde(skip)]
    pub recording: Option<Recording>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateStateRequest {
    #[garde(custom(is_valid_state))]
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct AlarmControlRequest {
    /// Action to perform: "on" (trigger), "off"/"cancel" (cancel), "status" (get status)
    #[garde(custom(is_valid_alarm_action))]
    pub action: String,

    /// Alarm score (1-100+). Only used when action is "on". Default: 100
    #[garde(skip)]
    #[serde(default)]
    pub score: Option<u32>,

    /// Short cause string (max 31 chars). Only used when action is "on". Default: "API"
    #[garde(length(max = 31))]
    #[serde(default)]
    pub cause: Option<String>,

    /// Description text (max 255 chars). Only used when action is "on".
    #[garde(length(max = 255))]
    #[serde(default)]
    pub text: Option<String>,
}

#[cfg(test)]
mod validator_tests {
    use super::*;

    #[test]
    fn url_component_rejects_control_chars_and_crlf() {
        assert!(is_safe_url_component("foo\nbar", &()).is_err());
        assert!(is_safe_url_component("foo\0bar", &()).is_err());
        assert!(is_safe_url_component("foo\rbar", &()).is_err());
        assert!(is_safe_url_component("foo\x1bbar", &()).is_err());
        assert!(is_safe_url_component("foo\x7fbar", &()).is_err());
    }

    #[test]
    fn url_component_accepts_typical_camera_values() {
        assert!(is_safe_url_component("192.168.1.50", &()).is_ok());
        assert!(is_safe_url_component("cam.local:8080", &()).is_ok());
        assert!(is_safe_url_component("/Streaming/Channels/101", &()).is_ok());
        assert!(is_safe_url_component("", &()).is_ok());
    }

    #[test]
    fn url_component_enforces_length_cap() {
        let too_long = "a".repeat(2049);
        assert!(is_safe_url_component(&too_long, &()).is_err());
        let at_cap = "a".repeat(2048);
        assert!(is_safe_url_component(&at_cap, &()).is_ok());
    }

    #[test]
    fn onvif_url_rejects_dangerous_schemes() {
        // The threat is SSRF from ZoneMinder's capture daemon making
        // outbound requests via the URL, so block schemes that resolve to
        // local FS, Unix sockets, or other gadgets. `javascript:` is
        // intentionally not covered here — the daemon won't render it; that
        // class belongs to whoever renders the URL in the UI.
        assert!(is_safe_onvif_url("file:///etc/passwd", &()).is_err());
        assert!(is_safe_onvif_url("unix:///var/run/foo.sock", &()).is_err());
        assert!(is_safe_onvif_url("gopher://attacker/x", &()).is_err());
        assert!(is_safe_onvif_url("ftp://example.com/", &()).is_err());
    }

    #[test]
    fn onvif_url_accepts_camera_schemes() {
        assert!(is_safe_onvif_url("http://192.168.1.50/onvif", &()).is_ok());
        assert!(is_safe_onvif_url("HTTPS://CAM.LOCAL:8080/x", &()).is_ok());
        assert!(is_safe_onvif_url("rtsp://cam:554/stream", &()).is_ok());
        assert!(is_safe_onvif_url("rtsps://cam:443/stream", &()).is_ok());
        // No scheme at all is also fine — the field has historically accepted
        // bare host:port style strings.
        assert!(is_safe_onvif_url("192.168.1.50", &()).is_ok());
        assert!(is_safe_onvif_url("", &()).is_ok());
    }

    #[test]
    fn command_string_rejects_nul_and_newlines_but_keeps_shell_metas() {
        // Shell metacharacters stay legal — the field IS designed to be
        // executed as a shell command by ZoneMinder.
        assert!(is_safe_command_string("echo hi | tee /tmp/log", &()).is_ok());
        assert!(is_safe_command_string("$(curl http://example.com)", &()).is_ok());
        assert!(is_safe_command_string("kill -9 $(pidof zmc)", &()).is_ok());
        assert!(is_safe_command_string("", &()).is_ok());

        // NUL + CRLF are rejected: they enable C-string truncation and log
        // forgery in the daemon that runs the command.
        assert!(is_safe_command_string("rm -rf /\n", &()).is_err());
        assert!(is_safe_command_string("echo \0", &()).is_err());
        assert!(is_safe_command_string("echo \rbad", &()).is_err());
    }

    #[test]
    fn command_string_enforces_length_cap() {
        let too_long = "a".repeat(4097);
        assert!(is_safe_command_string(&too_long, &()).is_err());
    }

    #[test]
    fn create_monitor_request_default_passes_validation() {
        // The Default impl must satisfy every `garde` constraint, so onboarding
        // (and any other caller) can build a valid monitor by overriding only a
        // few fields.
        super::CreateMonitorRequest::default()
            .validate()
            .expect("default CreateMonitorRequest must pass validation");
    }
}
