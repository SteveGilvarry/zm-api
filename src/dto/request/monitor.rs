// src/dto/request/monitor.rs
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
// // use sea_orm::ActiveEnum; // Unused import // Unused import
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
    pub enabled: u8,

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
    pub janus_use_rtsp_restream: i8,

    #[garde(skip)] // Option<i32> can be None
    pub janus_rtsp_user: Option<i32>,

    #[garde(skip)] // Option<i32> can be None
    pub janus_rtsp_session_timeout: Option<i32>,

    #[garde(skip)] // Option<String> can be None
    pub linked_monitors: Option<String>,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub triggers: String,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub event_start_command: String,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub event_end_command: String,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
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

    #[garde(skip)] // Option<String> can be None
    pub host: Option<String>,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub port: String,

    #[garde(length(min = 0))] // Empty string is valid, but validate it's a string
    pub sub_path: String,

    #[garde(skip)] // Option<String> can be None
    pub path: Option<String>,

    #[garde(skip)] // Option<String> can be None
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

    #[garde(skip)] // u16 doesn't need validation in this case
    pub default_rate: u16,

    #[garde(skip)] // u16 doesn't need validation in this case
    pub default_scale: u16,

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
    pub enabled: Option<u8>,

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
    pub janus_use_rtsp_restream: Option<i8>,

    #[garde(skip)]
    pub janus_rtsp_user: Option<i32>,

    #[garde(skip)]
    pub janus_rtsp_session_timeout: Option<i32>,

    #[garde(skip)]
    pub linked_monitors: Option<String>,

    #[garde(length(min = 0))]
    pub triggers: Option<String>,

    #[garde(length(min = 0))]
    pub event_start_command: Option<String>,

    #[garde(length(min = 0))]
    pub event_end_command: Option<String>,

    #[garde(length(min = 0))]
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

    #[garde(skip)]
    pub host: Option<String>,

    #[garde(length(min = 0))]
    pub port: Option<String>,

    #[garde(length(min = 0))]
    pub sub_path: Option<String>,

    #[garde(skip)]
    pub path: Option<String>,

    #[garde(skip)]
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

    #[garde(skip)]
    pub default_scale: Option<u16>,

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
    #[garde(custom(is_valid_alarm_action))]
    pub action: String,
}
