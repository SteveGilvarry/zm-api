#![allow(clippy::result_large_err)]
use tracing::info;
use crate::error::{AppResult, AppError, Resource, ResourceType};
use crate::server::state::AppState;
use crate::dto::request::{CreateMonitorRequest, UpdateMonitorRequest, UpdateStateRequest, AlarmControlRequest};
use crate::dto::response::{MonitorResponse, MonitorStreamingDetails};
use crate::entity::monitors;
use crate::repo;
use sea_orm::ActiveValue::Set;
use rust_decimal::prelude::*;
use url::Url;

pub async fn list_all(state: &AppState) -> AppResult<Vec<MonitorResponse>> {
    info!("Listing all monitors.");
    let monitors = repo::monitors::find_all(state.db()).await?;
    Ok(monitors.into_iter().map(MonitorResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<MonitorResponse> {
    info!("Getting monitor by ID: {id}.");
    let monitor = repo::monitors::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("id".to_string(), id.to_string())],
                resource_type: ResourceType::File, // Use appropriate resource type
            })
        })?;
    Ok(MonitorResponse::from(monitor))
}

pub async fn create(state: &AppState, req: CreateMonitorRequest) -> AppResult<MonitorResponse> {
    info!("Creating new monitor with request: {req:?}.");
    
    // Convert string values to their corresponding enum types
    let monitor_type =req.r#type.clone();
    let function = req.function.clone();
    let capturing = req.capturing.clone();
    let decoding = req.decoding.clone();
    let rtsp2_web_type = req.rtsp2_web_type.clone();
    let orientation = req.orientation.clone();
    let output_container = req.output_container.clone();
    let recording_source = req.recording_source.clone();
    let default_codec = req.default_codec.clone();
    let importance = req.importance.clone();
    let analysing = req.analysing.clone();
    let analysis_source = req.analysis_source.clone();
    let analysis_image = req.analysis_image.clone();
    let recording = req.recording.clone();
    
    // Create a new ActiveModel for the monitor
    let monitor = monitors::ActiveModel {
        id: Set(0), // This will be auto-generated
        name: Set(req.name),
        deleted: Set(req.deleted),
        notes: Set(req.notes),
        server_id: Set(req.server_id),
        storage_id: Set(req.storage_id),
        manufacturer_id: Set(req.manufacturer_id),
        model_id: Set(req.model_id),
        r#type: Set(monitor_type),
        function: Set(function),
        capturing: Set(capturing),
        enabled: Set(req.enabled),
        decoding_enabled: Set(req.decoding_enabled),
        decoding: Set(decoding),
        rtsp2_web_enabled: Set(req.rtsp2_web_enabled),
        rtsp2_web_type: Set(rtsp2_web_type),
        janus_enabled: Set(req.janus_enabled),
        janus_audio_enabled: Set(req.janus_audio_enabled),
        janus_profile_override: Set(req.janus_profile_override),
        janus_use_rtsp_restream: Set(req.janus_use_rtsp_restream),
        janus_rtsp_user: Set(req.janus_rtsp_user),
        janus_rtsp_session_timeout: Set(req.janus_rtsp_session_timeout),
        linked_monitors: Set(req.linked_monitors),
        triggers: Set(req.triggers),
        event_start_command: Set(req.event_start_command),
        event_end_command: Set(req.event_end_command),
        onvif_url: Set(req.onvif_url),
        onvif_events_path: Set(req.onvif_events_path),
        onvif_username: Set(req.onvif_username),
        onvif_password: Set(req.onvif_password),
        onvif_options: Set(req.onvif_options),
        onvif_event_listener: Set(req.onvif_event_listener),
        onvif_alarm_text: Set(req.onvif_alarm_text),
        use_amcrest_api: Set(req.use_amcrest_api),
        device: Set(req.device),
        channel: Set(req.channel),
        format: Set(req.format),
        v4l_multi_buffer: Set(req.v4l_multi_buffer),
        v4l_captures_per_frame: Set(req.v4l_captures_per_frame),
        protocol: Set(req.protocol),
        method: Set(req.method),
        host: Set(req.host),
        port: Set(req.port),
        sub_path: Set(req.sub_path),
        path: Set(req.path),
        second_path: Set(req.second_path),
        options: Set(req.options),
        user: Set(req.user),
        pass: Set(req.pass),
        width: Set(req.width),
        height: Set(req.height),
        colours: Set(req.colours),
        palette: Set(req.palette),
        orientation: Set(orientation),
        deinterlacing: Set(req.deinterlacing),
        decoder: Set(req.decoder),
        decoder_hw_accel_name: Set(req.decoder_hw_accel_name),
        decoder_hw_accel_device: Set(req.decoder_hw_accel_device),
        save_jpe_gs: Set(req.save_jpe_gs),
        video_writer: Set(req.video_writer),
        output_codec: Set(req.output_codec),
        encoder: Set(req.encoder),
        output_container: Set(Some(output_container)),
        encoder_parameters: Set(req.encoder_parameters),
        record_audio: Set(req.record_audio),
        recording_source: Set(recording_source),
        rtsp_describe: Set(req.rtsp_describe),
        brightness: Set(Some(req.brightness)),
        contrast: Set(Some(req.contrast)),
        hue: Set(Some(req.hue)),
        colour: Set(Some(req.colour)),
        event_prefix: Set(req.event_prefix),
        label_format: Set(req.label_format),
        label_x: Set(req.label_x),
        label_y: Set(req.label_y),
        label_size: Set(req.label_size),
        image_buffer_count: Set(req.image_buffer_count),
        max_image_buffer_count: Set(req.max_image_buffer_count),
        warmup_count: Set(req.warmup_count),
        pre_event_count: Set(req.pre_event_count),
        post_event_count: Set(req.post_event_count),
        stream_replay_buffer: Set(req.stream_replay_buffer),
        alarm_frame_count: Set(req.alarm_frame_count),
        section_length: Set(req.section_length),
        section_length_warn: Set(req.section_length_warn),
        event_close_mode: Set(req.event_close_mode),
        min_section_length: Set(req.min_section_length),
        frame_skip: Set(req.frame_skip),
        motion_frame_skip: Set(req.motion_frame_skip),
        analysis_fps_limit: Set(req.analysis_fps_limit.map(|f| Decimal::from_f64(f).unwrap_or_default())),
        analysis_update_delay: Set(req.analysis_update_delay),
        max_fps: Set(req.max_fps.map(|f| Decimal::from_f64(f).unwrap_or_default())),
        alarm_max_fps: Set(req.alarm_max_fps.map(|f| Decimal::from_f64(f).unwrap_or_default())),
        fps_report_interval: Set(req.fps_report_interval),
        ref_blend_perc: Set(req.ref_blend_perc),
        alarm_ref_blend_perc: Set(req.alarm_ref_blend_perc),
        controllable: Set(req.controllable),
        control_id: Set(req.control_id),
        control_device: Set(req.control_device),
        control_address: Set(req.control_address),
        auto_stop_timeout: Set(req.auto_stop_timeout.map(|f| Decimal::from_f64(f).unwrap_or_default())),
        track_motion: Set(req.track_motion),
        track_delay: Set(req.track_delay),
        return_location: Set(req.return_location),
        return_delay: Set(req.return_delay),
        modect_during_ptz: Set(req.modect_during_ptz),
        default_rate: Set(req.default_rate),
        default_scale: Set(req.default_scale),
        default_codec: Set(default_codec),
        signal_check_points: Set(req.signal_check_points),
        signal_check_colour: Set(req.signal_check_colour),
        web_colour: Set(req.web_colour),
        exif: Set(req.exif),
        sequence: Set(req.sequence),
        zone_count: Set(req.zone_count),
        refresh: Set(req.refresh),
        latitude: Set(req.latitude.map(|f| Decimal::from_f64(f).unwrap_or_default())),
        longitude: Set(req.longitude.map(|f| Decimal::from_f64(f).unwrap_or_default())),
        rtsp_server: Set(req.rtsp_server),
        rtsp_stream_name: Set(req.rtsp_stream_name),
        soap_wsa_compl: Set(req.soap_wsa_compl),
        importance: Set(importance),
        mqtt_enabled: Set(req.mqtt_enabled),
        mqtt_subscriptions: Set(req.mqtt_subscriptions),
        startup_delay: Set(req.startup_delay),
        analysing: Set(analysing),
        analysis_source: Set(analysis_source),
        analysis_image: Set(analysis_image),
        recording: Set(recording),
    };
    
    // Use the repository to insert the new monitor
    let result = repo::monitors::create(state.db(), monitor).await?;
    
    Ok(MonitorResponse::from(result))
}

pub async fn update(state: &AppState, id: u32, req: UpdateMonitorRequest) -> AppResult<MonitorResponse> {
    info!("Updating monitor with ID: {id} and request: {req:?}.");
    
    // Fetch the monitor through the repository
    let monitor_model = repo::monitors::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("id".to_string(), id.to_string())],
                resource_type: ResourceType::File,
            })
        })?;
        
    let mut monitor: monitors::ActiveModel = monitor_model.into();

    // Update fields if they are provided in the request
    if let Some(name) = req.name {
        monitor.name = Set(name);
    }
    if let Some(deleted) = req.deleted {
        monitor.deleted = Set(deleted != 0);
    }
    if req.notes.is_some() {
        monitor.notes = Set(req.notes);
    }
    if req.server_id.is_some() {
        monitor.server_id = Set(req.server_id);
    }
    if let Some(storage_id) = req.storage_id {
        monitor.storage_id = Set(storage_id);
    }
    if req.manufacturer_id.is_some() {
        monitor.manufacturer_id = Set(req.manufacturer_id);
    }
    if req.model_id.is_some() {
        monitor.model_id = Set(req.model_id);
    }
    if let Some(monitor_type) = req.r#type {
        monitor.r#type = Set(monitor_type);
    }
    if let Some(function) = req.function {
        monitor.function = Set(function);
    }
    if let Some(capturing) = req.capturing {
        monitor.capturing = Set(capturing);
    }
    if let Some(enabled) = req.enabled {
        monitor.enabled = Set(enabled);
    }
    if let Some(decoding_enabled) = req.decoding_enabled {
        monitor.decoding_enabled = Set(decoding_enabled);
    }
    if let Some(decoding) = req.decoding {
        monitor.decoding = Set(decoding);
    }
    if let Some(rtsp2_web_enabled) = req.rtsp2_web_enabled {
        monitor.rtsp2_web_enabled = Set(rtsp2_web_enabled);
    }
    if let Some(rtsp2_web_type) = req.rtsp2_web_type {
        monitor.rtsp2_web_type = Set(rtsp2_web_type);
    }
    if let Some(janus_enabled) = req.janus_enabled {
        monitor.janus_enabled = Set(janus_enabled);
    }
    if let Some(janus_audio_enabled) = req.janus_audio_enabled {
        monitor.janus_audio_enabled = Set(janus_audio_enabled);
    }
    if req.janus_profile_override.is_some() {
        monitor.janus_profile_override = Set(req.janus_profile_override);
    }
    if let Some(janus_use_rtsp_restream) = req.janus_use_rtsp_restream {
        monitor.janus_use_rtsp_restream = Set(janus_use_rtsp_restream);
    }
    if req.janus_rtsp_user.is_some() {
        monitor.janus_rtsp_user = Set(req.janus_rtsp_user);
    }
    if req.janus_rtsp_session_timeout.is_some() {
        monitor.janus_rtsp_session_timeout = Set(req.janus_rtsp_session_timeout);
    }
    if req.linked_monitors.is_some() {
        monitor.linked_monitors = Set(req.linked_monitors);
    }
    if let Some(triggers) = req.triggers {
        monitor.triggers = Set(triggers);
    }
    if let Some(event_start_command) = req.event_start_command {
        monitor.event_start_command = Set(event_start_command);
    }
    if let Some(event_end_command) = req.event_end_command {
        monitor.event_end_command = Set(event_end_command);
    }
    if let Some(onvif_url) = req.onvif_url {
        monitor.onvif_url = Set(onvif_url);
    }
    if let Some(onvif_events_path) = req.onvif_events_path {
        monitor.onvif_events_path = Set(onvif_events_path);
    }
    if let Some(onvif_username) = req.onvif_username {
        monitor.onvif_username = Set(onvif_username);
    }
    if let Some(onvif_password) = req.onvif_password {
        monitor.onvif_password = Set(onvif_password);
    }
    if let Some(onvif_options) = req.onvif_options {
        monitor.onvif_options = Set(onvif_options);
    }
    if let Some(onvif_event_listener) = req.onvif_event_listener {
        monitor.onvif_event_listener = Set(onvif_event_listener);
    }
    if req.onvif_alarm_text.is_some() {
        monitor.onvif_alarm_text = Set(req.onvif_alarm_text);
    }
    if let Some(use_amcrest_api) = req.use_amcrest_api {
        monitor.use_amcrest_api = Set(use_amcrest_api);
    }
    if let Some(device) = req.device {
        monitor.device = Set(device);
    }
    if let Some(channel) = req.channel {
        monitor.channel = Set(channel);
    }
    if let Some(format) = req.format {
        monitor.format = Set(format);
    }
    if req.v4l_multi_buffer.is_some() {
        monitor.v4l_multi_buffer = Set(req.v4l_multi_buffer);
    }
    if req.v4l_captures_per_frame.is_some() {
        monitor.v4l_captures_per_frame = Set(req.v4l_captures_per_frame);
    }
    if req.protocol.is_some() {
        monitor.protocol = Set(req.protocol);
    }
    if req.method.is_some() {
        monitor.method = Set(req.method);
    }
    if req.host.is_some() {
        monitor.host = Set(req.host);
    }
    if let Some(port) = req.port {
        monitor.port = Set(port);
    }
    if let Some(sub_path) = req.sub_path {
        monitor.sub_path = Set(sub_path);
    }
    if req.path.is_some() {
        monitor.path = Set(req.path);
    }
    if req.second_path.is_some() {
        monitor.second_path = Set(req.second_path);
    }
    if req.options.is_some() {
        monitor.options = Set(req.options);
    }
    if req.user.is_some() {
        monitor.user = Set(req.user);
    }
    if req.pass.is_some() {
        monitor.pass = Set(req.pass);
    }
    if let Some(width) = req.width {
        monitor.width = Set(width);
    }
    if let Some(height) = req.height {
        monitor.height = Set(height);
    }
    if let Some(colours) = req.colours {
        monitor.colours = Set(colours);
    }
    if let Some(palette) = req.palette {
        monitor.palette = Set(palette);
    }
    if let Some(orientation) = req.orientation {
        monitor.orientation = Set(orientation);
    }
    if let Some(deinterlacing) = req.deinterlacing {
        monitor.deinterlacing = Set(deinterlacing);
    }
    if req.decoder.is_some() {
        monitor.decoder = Set(req.decoder);
    }
    if req.decoder_hw_accel_name.is_some() {
        monitor.decoder_hw_accel_name = Set(req.decoder_hw_accel_name);
    }
    if req.decoder_hw_accel_device.is_some() {
        monitor.decoder_hw_accel_device = Set(req.decoder_hw_accel_device);
    }
    if let Some(save_jpe_gs) = req.save_jpe_gs {
        monitor.save_jpe_gs = Set(save_jpe_gs);
    }
    if let Some(video_writer) = req.video_writer {
        monitor.video_writer = Set(video_writer);
    }
    if req.output_codec.is_some() {
        monitor.output_codec = Set(req.output_codec);
    }
    if req.encoder.is_some() {
        monitor.encoder = Set(req.encoder);
    }
    if let Some(output_container) = req.output_container {
        monitor.output_container = Set(Some(output_container));
    }
    if req.encoder_parameters.is_some() {
        monitor.encoder_parameters = Set(req.encoder_parameters);
    }
    if let Some(record_audio) = req.record_audio {
        monitor.record_audio = Set(record_audio);
    }
    if let Some(recording_source) = req.recording_source {
        monitor.recording_source = Set(recording_source);
    }
    if req.rtsp_describe.is_some() {
        monitor.rtsp_describe = Set(req.rtsp_describe);
    }
    if let Some(brightness) = req.brightness {
        monitor.brightness = Set(Some(brightness));
    }
    if let Some(contrast) = req.contrast {
        monitor.contrast = Set(Some(contrast));
    }
    if let Some(hue) = req.hue {
        monitor.hue = Set(Some(hue));
    }
    if let Some(colour) = req.colour {
        monitor.colour = Set(Some(colour));
    }
    if let Some(event_prefix) = req.event_prefix {
        monitor.event_prefix = Set(event_prefix);
    }
    if req.label_format.is_some() {
        monitor.label_format = Set(req.label_format);
    }
    if let Some(label_x) = req.label_x {
        monitor.label_x = Set(label_x);
    }
    if let Some(label_y) = req.label_y {
        monitor.label_y = Set(label_y);
    }
    if let Some(label_size) = req.label_size {
        monitor.label_size = Set(label_size);
    }
    if let Some(image_buffer_count) = req.image_buffer_count {
        monitor.image_buffer_count = Set(image_buffer_count);
    }
    if let Some(max_image_buffer_count) = req.max_image_buffer_count {
        monitor.max_image_buffer_count = Set(max_image_buffer_count);
    }
    if let Some(warmup_count) = req.warmup_count {
        monitor.warmup_count = Set(warmup_count);
    }
    if let Some(pre_event_count) = req.pre_event_count {
        monitor.pre_event_count = Set(pre_event_count);
    }
    if let Some(post_event_count) = req.post_event_count {
        monitor.post_event_count = Set(post_event_count);
    }
    if let Some(stream_replay_buffer) = req.stream_replay_buffer {
        monitor.stream_replay_buffer = Set(stream_replay_buffer);
    }
    if let Some(alarm_frame_count) = req.alarm_frame_count {
        monitor.alarm_frame_count = Set(alarm_frame_count);
    }
    if let Some(section_length) = req.section_length {
        monitor.section_length = Set(section_length);
    }
    if let Some(section_length_warn) = req.section_length_warn {
        monitor.section_length_warn = Set(section_length_warn);
    }
    if let Some(event_close_mode) = req.event_close_mode {
        monitor.event_close_mode = Set(event_close_mode);
    }
    if let Some(min_section_length) = req.min_section_length {
        monitor.min_section_length = Set(min_section_length);
    }
    if let Some(frame_skip) = req.frame_skip {
        monitor.frame_skip = Set(frame_skip);
    }
    if let Some(motion_frame_skip) = req.motion_frame_skip {
        monitor.motion_frame_skip = Set(motion_frame_skip);
    }
    if let Some(analysis_fps_limit) = req.analysis_fps_limit {
        monitor.analysis_fps_limit = Set(Some(Decimal::from_f64(analysis_fps_limit).unwrap_or_default()));
    }
    if let Some(analysis_update_delay) = req.analysis_update_delay {
        monitor.analysis_update_delay = Set(analysis_update_delay);
    }
    if let Some(max_fps) = req.max_fps {
        monitor.max_fps = Set(Some(Decimal::from_f64(max_fps).unwrap_or_default()));
    }
    if let Some(alarm_max_fps) = req.alarm_max_fps {
        monitor.alarm_max_fps = Set(Some(Decimal::from_f64(alarm_max_fps).unwrap_or_default()));
    }
    if let Some(fps_report_interval) = req.fps_report_interval {
        monitor.fps_report_interval = Set(fps_report_interval);
    }
    if let Some(ref_blend_perc) = req.ref_blend_perc {
        monitor.ref_blend_perc = Set(ref_blend_perc);
    }
    if let Some(alarm_ref_blend_perc) = req.alarm_ref_blend_perc {
        monitor.alarm_ref_blend_perc = Set(alarm_ref_blend_perc);
    }
    if let Some(controllable) = req.controllable {
        monitor.controllable = Set(controllable);
    }
    if req.control_id.is_some() {
        monitor.control_id = Set(req.control_id);
    }
    if req.control_device.is_some() {
        monitor.control_device = Set(req.control_device);
    }
    if req.control_address.is_some() {
        monitor.control_address = Set(req.control_address);
    }
    if let Some(auto_stop_timeout) = req.auto_stop_timeout {
        monitor.auto_stop_timeout = Set(Some(Decimal::from_f64(auto_stop_timeout).unwrap_or_default()));
    }
    if let Some(track_motion) = req.track_motion {
        monitor.track_motion = Set(track_motion);
    }
    if req.track_delay.is_some() {
        monitor.track_delay = Set(req.track_delay);
    }
    if let Some(return_location) = req.return_location {
        monitor.return_location = Set(return_location);
    }
    if req.return_delay.is_some() {
        monitor.return_delay = Set(req.return_delay);
    }
    if let Some(modect_during_ptz) = req.modect_during_ptz {
        monitor.modect_during_ptz = Set(modect_during_ptz);
    }
    if let Some(default_rate) = req.default_rate {
        monitor.default_rate = Set(default_rate);
    }
    if let Some(default_scale) = req.default_scale {
        monitor.default_scale = Set(default_scale);
    }
    if let Some(default_codec) = req.default_codec {
        monitor.default_codec = Set(default_codec);
    }
    if let Some(signal_check_points) = req.signal_check_points {
        monitor.signal_check_points = Set(signal_check_points);
    }
    if let Some(signal_check_colour) = req.signal_check_colour {
        monitor.signal_check_colour = Set(signal_check_colour);
    }
    if let Some(web_colour) = req.web_colour {
        monitor.web_colour = Set(web_colour);
    }
    if let Some(exif) = req.exif {
        monitor.exif = Set(exif);
    }
    if req.sequence.is_some() {
        monitor.sequence = Set(req.sequence);
    }
    if let Some(zone_count) = req.zone_count {
        monitor.zone_count = Set(zone_count);
    }
    if req.refresh.is_some() {
        monitor.refresh = Set(req.refresh);
    }
    if let Some(latitude) = req.latitude {
        monitor.latitude = Set(Some(Decimal::from_f64(latitude).unwrap_or_default()));
    }
    if let Some(longitude) = req.longitude {
        monitor.longitude = Set(Some(Decimal::from_f64(longitude).unwrap_or_default()));
    }
    if let Some(rtsp_server) = req.rtsp_server {
        monitor.rtsp_server = Set(rtsp_server);
    }
    if let Some(rtsp_stream_name) = req.rtsp_stream_name {
        monitor.rtsp_stream_name = Set(rtsp_stream_name);
    }
    if let Some(soap_wsa_compl) = req.soap_wsa_compl {
        monitor.soap_wsa_compl = Set(soap_wsa_compl);
    }
    if let Some(importance) = req.importance {
        monitor.importance = Set(importance);
    }
    if let Some(mqtt_enabled) = req.mqtt_enabled {
        monitor.mqtt_enabled = Set(mqtt_enabled);
    }
    if let Some(mqtt_subscriptions) = req.mqtt_subscriptions {
        monitor.mqtt_subscriptions = Set(mqtt_subscriptions);
    }
    if let Some(startup_delay) = req.startup_delay {
        monitor.startup_delay = Set(startup_delay);
    }
    if let Some(analysing) = req.analysing {
        monitor.analysing = Set(analysing);
    }
    if let Some(analysis_source) = req.analysis_source {
        monitor.analysis_source = Set(analysis_source);
    }
    if let Some(analysis_image) = req.analysis_image {
        monitor.analysis_image = Set(analysis_image);
    }
    if let Some(recording) = req.recording {
        monitor.recording = Set(recording);
    }

    // Use the repository to update the monitor
    let updated_monitor = repo::monitors::update(state.db(), monitor).await?;
    Ok(MonitorResponse::from(updated_monitor))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    info!("Deleting monitor with ID: {id}.");
    let monitor = repo::monitors::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("id".to_string(), id.to_string())],
                resource_type: ResourceType::File,
            })
        })?;
    repo::monitors::delete(state.db(), monitor).await
}

pub async fn update_state(state: &AppState, id: u32, req: UpdateStateRequest) -> AppResult<MonitorResponse> {
    info!("Updating state of monitor with ID: {id} and request: {req:?}.");
    
    // Use repository to fetch the monitor
    let monitor_model = repo::monitors::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("id".to_string(), id.to_string())],
                resource_type: ResourceType::Monitor,
            })
        })?;
    
    let mut monitor: monitors::ActiveModel = monitor_model.clone().into();
    
    // Update the monitor state based on the request
    match req.state.as_str() {
        "start" => {
            // Enable the monitor
            monitor.enabled = Set(1);
            // TODO: Additional logic to actually start the monitor via ZoneMinder API
        },
        "stop" => {
            // Disable the monitor
            monitor.enabled = Set(0);
            // TODO: Additional logic to actually stop the monitor via ZoneMinder API
        },
        "restart" => {
            // No change to enabled state, it will be restarted by ZoneMinder
            // TODO: Additional logic to actually restart the monitor via ZoneMinder API
        },
        _ => {
            return Err(AppError::BadRequestError(format!("Invalid state: {}", req.state)));
        }
    }
    
    // Use repository to update the monitor
    let updated_monitor = repo::monitors::update(state.db(), monitor).await?;
    
    // Return the updated monitor
    Ok(MonitorResponse::from(updated_monitor))
}

/// Get streaming connection details for a monitor
pub async fn get_streaming_details(state: &AppState, monitor_id: u32) -> AppResult<MonitorStreamingDetails> {
    let monitor = repo::monitors::get_streaming_details(state.db(), monitor_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("id".to_string(), monitor_id.to_string())],
                resource_type: ResourceType::Monitor,
            })
        })?;
    
    // Extract user and password
    let user = monitor.user.clone().unwrap_or_default();
    let pass = monitor.pass.clone().unwrap_or_default();
    
    // Extract host and port from the path field
    let (host, port) = parse_host_port(&monitor.path)?;
    
    Ok(MonitorStreamingDetails {
        id: monitor.id,
        name: monitor.name.clone(),
        user,
        pass,
        host,
        port,
    })
}

/// Parse the host and port from a URL string
fn parse_host_port(url_str: &Option<String>) -> AppResult<(String, u16)> {
    let default_port = 80;
    
    match url_str {
        Some(url_string) if !url_string.is_empty() => {
            // Try to parse as URL
            match Url::parse(url_string) {
                Ok(url) => {
                    if let Some(host) = url.host_str() {
                        let port = url.port().unwrap_or(default_port);
                        Ok((host.to_string(), port))
                    } else {
                        // Handle inputs like "host:port" where Url::parse treated it as a scheme.
                        if url_string.contains("://") {
                            return Err(AppError::BadRequestError("URL is missing host".to_string()));
                        }
                        if let Some((host, port_str)) = url_string.rsplit_once(':') {
                            let port = port_str
                                .parse::<u16>()
                                .map_err(|_| AppError::BadRequestError("Invalid port number".to_string()))?;
                            Ok((host.to_string(), port))
                        } else {
                            Ok((url_string.to_string(), default_port))
                        }
                    }
                }
                Err(_) => {
                    // If URL parsing fails, try to extract host:port directly
                    if let Some((host, port_str)) = url_string.rsplit_once(':') {
                        let port = port_str
                            .parse::<u16>()
                            .map_err(|_| AppError::BadRequestError("Invalid port number".to_string()))?;
                        Ok((host.to_string(), port))
                    } else {
                        // If no port, return host with default port
                        Ok((url_string.to_string(), default_port))
                    }
                }
            }
        },
        _ => Err(AppError::BadRequestError("Missing or empty URL".to_string())),
    }
}

pub async fn control_alarm(state: &AppState, id: u32, req: AlarmControlRequest) -> AppResult<MonitorResponse> {
    info!("Controlling alarm of monitor with ID: {id} and request: {req:?}.");
    
    // Use repository to fetch the monitor
    let monitor_model = repo::monitors::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("id".to_string(), id.to_string())],
                resource_type: ResourceType::Monitor,
            })
        })?;
    
    // Handle the alarm action
    match req.action.as_str() {
        "on" => {
            // TODO: Set monitor to alarm state
            // This will require additional implementation to trigger an alarm in ZoneMinder
            // For example, creating an event in the Events table and/or
            // making an API call to ZoneMinder
        },
        "off" => {
            // TODO: Cancel alarm state
            // This will require additional implementation to cancel an active alarm
            // in ZoneMinder
        },
        "status" => {
            // TODO: Get current alarm status
            // This will require checking if there are any active events/alarms
            // for this monitor
        },
        _ => {
            return Err(AppError::BadRequestError(format!("Invalid alarm action: {}", req.action)));
        }
    }
    
    // For now, simply return the current monitor state
    // In a complete implementation, this would reflect the alarm status
    Ok(MonitorResponse::from(monitor_model))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host_port_variants() {
        // Host:port string
        let hp = Some("host.local:9090".to_string());
        let (h2, p2) = parse_host_port(&hp).unwrap();
        assert_eq!(h2, "host.local");
        assert_eq!(p2, 9090);

        // Bare host
        let host_only = Some("camera.lan".to_string());
        let (h4, p4) = parse_host_port(&host_only).unwrap();
        assert_eq!(h4, "camera.lan");
        assert_eq!(p4, 80);

        // Missing input
        let none: Option<String> = None;
        assert!(matches!(parse_host_port(&none).err().unwrap(), AppError::BadRequestError(_)));

        // Invalid port
        let bad_port = Some("camera.lan:notaport".to_string());
        assert!(matches!(parse_host_port(&bad_port).err().unwrap(), AppError::BadRequestError(_)));
    }
}
