
/*
 * -------------------------------------------------------
 * THIS FILE WAS AUTOMATICALLY GENERATED (DO NOT MODIFY)
 * -------------------------------------------------------
 */

/* tslint:disable */
/* eslint-disable */
export enum Monitor_Type {
    Local = "Local",
    Remote = "Remote",
    File = "File",
    Ffmpeg = "Ffmpeg",
    Libvlc = "Libvlc",
    cURL = "cURL",
    NVSocket = "NVSocket",
    VNC = "VNC"
}

export enum Monitors_Function {
    None = "None",
    Monitor = "Monitor",
    Modect = "Modect",
    Record = "Record",
    Mocord = "Mocord",
    Nodect = "Nodect"
}

export enum Monitors_Orientation {
    ROTATE_0 = "ROTATE_0",
    ROTATE_90 = "ROTATE_90",
    ROTATE_180 = "ROTATE_180",
    ROTATE_270 = "ROTATE_270",
    FLIP_HORI = "FLIP_HORI",
    FLIP_VERT = "FLIP_VERT"
}

export enum Monitors_OutputContainer {
    auto = "auto",
    mp4 = "mp4",
    mkv = "mkv"
}

export enum Monitors_DefaultCodec {
    auto = "auto",
    MP4 = "MP4",
    MJPEG = "MJPEG"
}

export enum Monitors_Importance {
    Not = "Not",
    Less = "Less",
    Normal = "Normal"
}

export class CreateConfigInput {
    Id?: Nullable<number>;
    Name?: Nullable<string>;
    Value?: Nullable<string>;
    Type?: Nullable<string>;
    DefaultValue?: Nullable<string>;
    Hint?: Nullable<string>;
    Pattern?: Nullable<string>;
    Format?: Nullable<string>;
    Prompt?: Nullable<string>;
    Help?: Nullable<string>;
    Category?: Nullable<string>;
    Readonly?: Nullable<boolean>;
    Requires?: Nullable<string>;
}

export class UpdateConfigInput {
    Id?: Nullable<number>;
    Name: string;
    Value?: Nullable<string>;
    Type?: Nullable<string>;
    DefaultValue?: Nullable<string>;
    Hint?: Nullable<string>;
    Pattern?: Nullable<string>;
    Format?: Nullable<string>;
    Prompt?: Nullable<string>;
    Help?: Nullable<string>;
    Category?: Nullable<string>;
    Readonly?: Nullable<boolean>;
    Requires?: Nullable<string>;
}

export class CreateControlpresetInput {
    MonitorId: number;
    Preset: number;
    Label?: Nullable<string>;
}

export class UpdateControlpresetInput {
    MonitorId: number;
    Preset: number;
    Label?: Nullable<string>;
}

export class CreateControlInput {
    Name?: Nullable<boolean>;
    Type?: Nullable<boolean>;
    Protocol?: Nullable<boolean>;
    CanWake?: Nullable<boolean>;
    CanSleep?: Nullable<boolean>;
    CanReset?: Nullable<boolean>;
    CanReboot?: Nullable<boolean>;
    CanZoom?: Nullable<boolean>;
    CanAutoZoom?: Nullable<boolean>;
    CanZoomAbs?: Nullable<boolean>;
    CanZoomRel?: Nullable<boolean>;
    CanZoomCon?: Nullable<boolean>;
    MinZoomRange?: Nullable<boolean>;
    MaxZoomRange?: Nullable<boolean>;
    MinZoomStep?: Nullable<boolean>;
    MaxZoomStep?: Nullable<boolean>;
    HasZoomSpeed?: Nullable<boolean>;
    MinZoomSpeed?: Nullable<boolean>;
    MaxZoomSpeed?: Nullable<boolean>;
    CanFocus?: Nullable<boolean>;
    CanAutoFocus?: Nullable<boolean>;
    CanFocusAbs?: Nullable<boolean>;
    CanFocusRel?: Nullable<boolean>;
    CanFocusCon?: Nullable<boolean>;
    MinFocusRange?: Nullable<boolean>;
    MaxFocusRange?: Nullable<boolean>;
    MinFocusStep?: Nullable<boolean>;
    MaxFocusStep?: Nullable<boolean>;
    HasFocusSpeed?: Nullable<boolean>;
    MinFocusSpeed?: Nullable<boolean>;
    MaxFocusSpeed?: Nullable<boolean>;
    CanIris?: Nullable<boolean>;
    CanAutoIris?: Nullable<boolean>;
    CanIrisAbs?: Nullable<boolean>;
    CanIrisRel?: Nullable<boolean>;
    CanIrisCon?: Nullable<boolean>;
    MinIrisRange?: Nullable<boolean>;
    MaxIrisRange?: Nullable<boolean>;
    MinIrisStep?: Nullable<boolean>;
    MaxIrisStep?: Nullable<boolean>;
    HasIrisSpeed?: Nullable<boolean>;
    MinIrisSpeed?: Nullable<boolean>;
    MaxIrisSpeed?: Nullable<boolean>;
    CanGain?: Nullable<boolean>;
    CanAutoGain?: Nullable<boolean>;
    CanGainAbs?: Nullable<boolean>;
    CanGainRel?: Nullable<boolean>;
    CanGainCon?: Nullable<boolean>;
    MinGainRange?: Nullable<boolean>;
    MaxGainRange?: Nullable<boolean>;
    MinGainStep?: Nullable<boolean>;
    MaxGainStep?: Nullable<boolean>;
    HasGainSpeed?: Nullable<boolean>;
    MinGainSpeed?: Nullable<boolean>;
    MaxGainSpeed?: Nullable<boolean>;
    CanWhite?: Nullable<boolean>;
    CanAutoWhite?: Nullable<boolean>;
    CanWhiteAbs?: Nullable<boolean>;
    CanWhiteRel?: Nullable<boolean>;
    CanWhiteCon?: Nullable<boolean>;
    MinWhiteRange?: Nullable<boolean>;
    MaxWhiteRange?: Nullable<boolean>;
    MinWhiteStep?: Nullable<boolean>;
    MaxWhiteStep?: Nullable<boolean>;
    HasWhiteSpeed?: Nullable<boolean>;
    MinWhiteSpeed?: Nullable<boolean>;
    MaxWhiteSpeed?: Nullable<boolean>;
    HasPresets?: Nullable<boolean>;
    NumPresets?: Nullable<boolean>;
    HasHomePreset?: Nullable<boolean>;
    CanSetPresets?: Nullable<boolean>;
    CanMove?: Nullable<boolean>;
    CanMoveDiag?: Nullable<boolean>;
    CanMoveMap?: Nullable<boolean>;
    CanMoveAbs?: Nullable<boolean>;
    CanMoveRel?: Nullable<boolean>;
    CanMoveCon?: Nullable<boolean>;
    CanPan?: Nullable<boolean>;
    MinPanRange?: Nullable<boolean>;
    MaxPanRange?: Nullable<boolean>;
    MinPanStep?: Nullable<boolean>;
    MaxPanStep?: Nullable<boolean>;
    HasPanSpeed?: Nullable<boolean>;
    MinPanSpeed?: Nullable<boolean>;
    MaxPanSpeed?: Nullable<boolean>;
    HasTurboPan?: Nullable<boolean>;
    TurboPanSpeed?: Nullable<boolean>;
    CanTilt?: Nullable<boolean>;
    MinTiltRange?: Nullable<boolean>;
    MaxTiltRange?: Nullable<boolean>;
    MinTiltStep?: Nullable<boolean>;
    MaxTiltStep?: Nullable<boolean>;
    HasTiltSpeed?: Nullable<boolean>;
    MinTiltSpeed?: Nullable<boolean>;
    MaxTiltSpeed?: Nullable<boolean>;
    HasTurboTilt?: Nullable<boolean>;
    TurboTiltSpeed?: Nullable<boolean>;
    CanAutoScan?: Nullable<boolean>;
    NumScanPaths?: Nullable<boolean>;
}

export class UpdateControlInput {
    id: number;
    Name?: Nullable<boolean>;
    Type?: Nullable<boolean>;
    Protocol?: Nullable<boolean>;
    CanWake?: Nullable<boolean>;
    CanSleep?: Nullable<boolean>;
    CanReset?: Nullable<boolean>;
    CanReboot?: Nullable<boolean>;
    CanZoom?: Nullable<boolean>;
    CanAutoZoom?: Nullable<boolean>;
    CanZoomAbs?: Nullable<boolean>;
    CanZoomRel?: Nullable<boolean>;
    CanZoomCon?: Nullable<boolean>;
    MinZoomRange?: Nullable<boolean>;
    MaxZoomRange?: Nullable<boolean>;
    MinZoomStep?: Nullable<boolean>;
    MaxZoomStep?: Nullable<boolean>;
    HasZoomSpeed?: Nullable<boolean>;
    MinZoomSpeed?: Nullable<boolean>;
    MaxZoomSpeed?: Nullable<boolean>;
    CanFocus?: Nullable<boolean>;
    CanAutoFocus?: Nullable<boolean>;
    CanFocusAbs?: Nullable<boolean>;
    CanFocusRel?: Nullable<boolean>;
    CanFocusCon?: Nullable<boolean>;
    MinFocusRange?: Nullable<boolean>;
    MaxFocusRange?: Nullable<boolean>;
    MinFocusStep?: Nullable<boolean>;
    MaxFocusStep?: Nullable<boolean>;
    HasFocusSpeed?: Nullable<boolean>;
    MinFocusSpeed?: Nullable<boolean>;
    MaxFocusSpeed?: Nullable<boolean>;
    CanIris?: Nullable<boolean>;
    CanAutoIris?: Nullable<boolean>;
    CanIrisAbs?: Nullable<boolean>;
    CanIrisRel?: Nullable<boolean>;
    CanIrisCon?: Nullable<boolean>;
    MinIrisRange?: Nullable<boolean>;
    MaxIrisRange?: Nullable<boolean>;
    MinIrisStep?: Nullable<boolean>;
    MaxIrisStep?: Nullable<boolean>;
    HasIrisSpeed?: Nullable<boolean>;
    MinIrisSpeed?: Nullable<boolean>;
    MaxIrisSpeed?: Nullable<boolean>;
    CanGain?: Nullable<boolean>;
    CanAutoGain?: Nullable<boolean>;
    CanGainAbs?: Nullable<boolean>;
    CanGainRel?: Nullable<boolean>;
    CanGainCon?: Nullable<boolean>;
    MinGainRange?: Nullable<boolean>;
    MaxGainRange?: Nullable<boolean>;
    MinGainStep?: Nullable<boolean>;
    MaxGainStep?: Nullable<boolean>;
    HasGainSpeed?: Nullable<boolean>;
    MinGainSpeed?: Nullable<boolean>;
    MaxGainSpeed?: Nullable<boolean>;
    CanWhite?: Nullable<boolean>;
    CanAutoWhite?: Nullable<boolean>;
    CanWhiteAbs?: Nullable<boolean>;
    CanWhiteRel?: Nullable<boolean>;
    CanWhiteCon?: Nullable<boolean>;
    MinWhiteRange?: Nullable<boolean>;
    MaxWhiteRange?: Nullable<boolean>;
    MinWhiteStep?: Nullable<boolean>;
    MaxWhiteStep?: Nullable<boolean>;
    HasWhiteSpeed?: Nullable<boolean>;
    MinWhiteSpeed?: Nullable<boolean>;
    MaxWhiteSpeed?: Nullable<boolean>;
    HasPresets?: Nullable<boolean>;
    NumPresets?: Nullable<boolean>;
    HasHomePreset?: Nullable<boolean>;
    CanSetPresets?: Nullable<boolean>;
    CanMove?: Nullable<boolean>;
    CanMoveDiag?: Nullable<boolean>;
    CanMoveMap?: Nullable<boolean>;
    CanMoveAbs?: Nullable<boolean>;
    CanMoveRel?: Nullable<boolean>;
    CanMoveCon?: Nullable<boolean>;
    CanPan?: Nullable<boolean>;
    MinPanRange?: Nullable<boolean>;
    MaxPanRange?: Nullable<boolean>;
    MinPanStep?: Nullable<boolean>;
    MaxPanStep?: Nullable<boolean>;
    HasPanSpeed?: Nullable<boolean>;
    MinPanSpeed?: Nullable<boolean>;
    MaxPanSpeed?: Nullable<boolean>;
    HasTurboPan?: Nullable<boolean>;
    TurboPanSpeed?: Nullable<boolean>;
    CanTilt?: Nullable<boolean>;
    MinTiltRange?: Nullable<boolean>;
    MaxTiltRange?: Nullable<boolean>;
    MinTiltStep?: Nullable<boolean>;
    MaxTiltStep?: Nullable<boolean>;
    HasTiltSpeed?: Nullable<boolean>;
    MinTiltSpeed?: Nullable<boolean>;
    MaxTiltSpeed?: Nullable<boolean>;
    HasTurboTilt?: Nullable<boolean>;
    TurboTiltSpeed?: Nullable<boolean>;
    CanAutoScan?: Nullable<boolean>;
    NumScanPaths?: Nullable<boolean>;
}

export class CreateDeviceInput {
    exampleField?: Nullable<number>;
}

export class UpdateDeviceInput {
    id: number;
}

export class CreateEventInput {
    exampleField?: Nullable<number>;
}

export class UpdateEventInput {
    id: number;
}

export class CreateEventsummaryInput {
    exampleField?: Nullable<number>;
}

export class UpdateEventsummaryInput {
    id: number;
}

export class CreateFilterInput {
    exampleField?: Nullable<number>;
}

export class UpdateFilterInput {
    id: number;
}

export class CreateFrameInput {
    exampleField?: Nullable<number>;
}

export class UpdateFrameInput {
    id: number;
}

export class CreateGroupInput {
    exampleField?: Nullable<number>;
}

export class UpdateGroupInput {
    id: number;
}

export class CreateLogInput {
    exampleField?: Nullable<number>;
}

export class UpdateLogInput {
    id: number;
}

export class CreateManufacturerInput {
    exampleField?: Nullable<number>;
}

export class UpdateManufacturerInput {
    id: number;
}

export class CreateModelInput {
    exampleField?: Nullable<number>;
}

export class UpdateModelInput {
    id: number;
}

export class CreateMonitorpresetInput {
    exampleField?: Nullable<number>;
}

export class UpdateMonitorpresetInput {
    id: number;
}

export class CreateMonitorInput {
    Name: string;
    Notes?: Nullable<string>;
    ServerId?: Nullable<number>;
    StorageId: number;
    Type: Monitor_Type;
    Function: Monitors_Function;
    Enabled: number;
    DecodingEnabled: number;
    LinkedMonitors?: Nullable<string>;
    Triggers: string;
    ONVIF_URL: string;
    ONVIF_Username: string;
    ONVIF_Password: string;
    ONVIF_Options: string;
    Device: string;
    Channel: number;
    Format: number;
    V4LMultiBuffer?: Nullable<number>;
    V4LCapturesPerFrame?: Nullable<number>;
    Protocol?: Nullable<string>;
    Method?: Nullable<string>;
    Host?: Nullable<string>;
    Port: string;
    SubPath: string;
    Path?: Nullable<string>;
    SecondPath?: Nullable<string>;
    Options?: Nullable<string>;
    User?: Nullable<string>;
    Pass?: Nullable<string>;
    Width: number;
    Height: number;
    Colours: number;
    Palette: number;
    Orientation?: Nullable<Monitors_Orientation>;
    Deinterlacing: number;
    DecoderHWAccelName?: Nullable<string>;
    DecoderHWAccelDevice?: Nullable<string>;
    SaveJPEGs: number;
    VideoWriter: number;
    OutputCodec?: Nullable<number>;
    Encoder?: Nullable<string>;
    OutputContainer?: Nullable<Monitors_OutputContainer>;
    EncoderParameters?: Nullable<string>;
    RecordAudio: number;
    RTSPDescribe?: Nullable<number>;
    Brightness: number;
    Contrast: number;
    Hue: number;
    Colour: number;
    EventPrefix: string;
    LabelFormat?: Nullable<string>;
    LabelX: number;
    LabelY: number;
    LabelSize: number;
    ImageBufferCount: number;
    MaxImageBufferCount: number;
    WarmupCount: number;
    PreEventCount: number;
    PostEventCount: number;
    StreamReplayBuffer: number;
    AlarmFrameCount: number;
    SectionLength: number;
    MinSectionLength: number;
    FrameSkip: number;
    MotionFrameSkip: number;
    AnalysisFPSLimit?: Nullable<number>;
    AnalysisUpdateDelay: number;
    MaxFPS?: Nullable<number>;
    AlarmMaxFPS?: Nullable<number>;
    FPSReportInterval: number;
    RefBlendPerc: number;
    AlarmRefBlendPerc: number;
    Controllable: number;
    ControlId?: Nullable<number>;
    ControlDevice?: Nullable<string>;
    ControlAddress?: Nullable<string>;
    AutoStopTimeout?: Nullable<number>;
    TrackMotion: number;
    TrackDelay?: Nullable<number>;
    ReturnLocation: number;
    ReturnDelay?: Nullable<number>;
    ModectDuringPTZ: number;
    DefaultRate: number;
    DefaultScale: number;
    DefaultCodec?: Nullable<Monitors_DefaultCodec>;
    SignalCheckPoints: number;
    SignalCheckColour: string;
    WebColour: string;
    Exif: number;
    Sequence?: Nullable<number>;
    TotalEvents?: Nullable<number>;
    ZoneCount: number;
    TotalEventDiskSpace?: Nullable<BigInt>;
    Refresh?: Nullable<number>;
    Latitude?: Nullable<number>;
    Longitude?: Nullable<number>;
    RTSPServer: boolean;
    RTSPStreamName: string;
    Importance?: Nullable<Monitors_Importance>;
    CreatedAt?: Nullable<DateTime>;
    UpdatedAt?: Nullable<DateTime>;
    LastChangeUser?: Nullable<string>;
}

export class UpdateMonitorInput {
    Id: number;
    Name: string;
    Notes?: Nullable<string>;
    ServerId?: Nullable<number>;
    StorageId: number;
    Type: Monitor_Type;
    Function: Monitors_Function;
    Enabled: number;
    DecodingEnabled: number;
    LinkedMonitors?: Nullable<string>;
    Triggers: string;
    ONVIF_URL: string;
    ONVIF_Username: string;
    ONVIF_Password: string;
    ONVIF_Options: string;
    Device: string;
    Channel: number;
    Format: number;
    V4LMultiBuffer?: Nullable<number>;
    V4LCapturesPerFrame?: Nullable<number>;
    Protocol?: Nullable<string>;
    Method?: Nullable<string>;
    Host?: Nullable<string>;
    Port: string;
    SubPath: string;
    Path?: Nullable<string>;
    SecondPath?: Nullable<string>;
    Options?: Nullable<string>;
    User?: Nullable<string>;
    Pass?: Nullable<string>;
    Width: number;
    Height: number;
    Colours: number;
    Palette: number;
    Orientation?: Nullable<Monitors_Orientation>;
    Deinterlacing: number;
    DecoderHWAccelName?: Nullable<string>;
    DecoderHWAccelDevice?: Nullable<string>;
    SaveJPEGs: number;
    VideoWriter: number;
    OutputCodec?: Nullable<number>;
    Encoder?: Nullable<string>;
    OutputContainer?: Nullable<Monitors_OutputContainer>;
    EncoderParameters?: Nullable<string>;
    RecordAudio: number;
    RTSPDescribe?: Nullable<number>;
    Brightness: number;
    Contrast: number;
    Hue: number;
    Colour: number;
    EventPrefix: string;
    LabelFormat?: Nullable<string>;
    LabelX: number;
    LabelY: number;
    LabelSize: number;
    ImageBufferCount: number;
    MaxImageBufferCount: number;
    WarmupCount: number;
    PreEventCount: number;
    PostEventCount: number;
    StreamReplayBuffer: number;
    AlarmFrameCount: number;
    SectionLength: number;
    MinSectionLength: number;
    FrameSkip: number;
    MotionFrameSkip: number;
    AnalysisFPSLimit?: Nullable<number>;
    AnalysisUpdateDelay: number;
    MaxFPS?: Nullable<number>;
    AlarmMaxFPS?: Nullable<number>;
    FPSReportInterval: number;
    RefBlendPerc: number;
    AlarmRefBlendPerc: number;
    Controllable: number;
    ControlId?: Nullable<number>;
    ControlDevice?: Nullable<string>;
    ControlAddress?: Nullable<string>;
    AutoStopTimeout?: Nullable<number>;
    TrackMotion: number;
    TrackDelay?: Nullable<number>;
    ReturnLocation: number;
    ReturnDelay?: Nullable<number>;
    ModectDuringPTZ: number;
    DefaultRate: number;
    DefaultScale: number;
    DefaultCodec?: Nullable<Monitors_DefaultCodec>;
    SignalCheckPoints: number;
    SignalCheckColour: string;
    WebColour: string;
    Exif: number;
    Sequence?: Nullable<number>;
    TotalEvents?: Nullable<number>;
    ZoneCount: number;
    TotalEventDiskSpace?: Nullable<BigInt>;
    Refresh?: Nullable<number>;
    Latitude?: Nullable<number>;
    Longitude?: Nullable<number>;
    RTSPServer: boolean;
    RTSPStreamName: string;
    Importance?: Nullable<Monitors_Importance>;
    CreatedAt?: Nullable<DateTime>;
    UpdatedAt?: Nullable<DateTime>;
    LastChangeUser?: Nullable<string>;
}

export class CreateMonitorstatusInput {
    exampleField?: Nullable<number>;
}

export class UpdateMonitorstatusInput {
    id: number;
}

export class CreateMontagelayoutInput {
    exampleField?: Nullable<number>;
}

export class UpdateMontagelayoutInput {
    id: number;
}

export class CreateServerInput {
    exampleField?: Nullable<number>;
}

export class UpdateServerInput {
    id: number;
}

export class CreateStateInput {
    exampleField?: Nullable<number>;
}

export class UpdateStateInput {
    id: number;
}

export class CreateStorageInput {
    exampleField?: Nullable<number>;
}

export class UpdateStorageInput {
    id: number;
}

export class CreateUserInput {
    Id?: Nullable<number>;
    Username?: Nullable<string>;
    Password?: Nullable<string>;
    Language?: Nullable<string>;
    Enabled?: Nullable<boolean>;
    MaxBandwidth?: Nullable<string>;
    TokenMinExpiry?: Nullable<BigInt>;
    APIEnabled?: Nullable<boolean>;
    HomeView?: Nullable<string>;
}

export class UpdateUserInput {
    Id?: Nullable<number>;
    Username?: Nullable<string>;
    Language?: Nullable<string>;
    Enabled?: Nullable<boolean>;
    MaxBandwidth?: Nullable<string>;
    TokenMinExpiry?: Nullable<BigInt>;
    APIEnabled?: Nullable<boolean>;
    HomeView?: Nullable<string>;
}

export class CreateZonepresetInput {
    exampleField?: Nullable<number>;
}

export class UpdateZonepresetInput {
    id: number;
}

export class CreateZoneInput {
    exampleField?: Nullable<number>;
}

export class UpdateZoneInput {
    id: number;
}

export class Config {
    Id?: Nullable<number>;
    Name?: Nullable<string>;
    Value?: Nullable<string>;
    Type?: Nullable<string>;
    DefaultValue?: Nullable<string>;
    Hint?: Nullable<string>;
    Pattern?: Nullable<string>;
    Format?: Nullable<string>;
    Prompt?: Nullable<string>;
    Help?: Nullable<string>;
    Category?: Nullable<string>;
    Readonly?: Nullable<boolean>;
    Requires?: Nullable<string>;
}

export abstract class IQuery {
    abstract configs(): Nullable<Config>[] | Promise<Nullable<Config>[]>;

    abstract config(Name: string): Nullable<Config> | Promise<Nullable<Config>>;

    abstract controlpresets(): Nullable<Controlpreset>[] | Promise<Nullable<Controlpreset>[]>;

    abstract controlpreset(monitorid: number, preset: number): Nullable<Controlpreset> | Promise<Nullable<Controlpreset>>;

    abstract controls(): Nullable<Control>[] | Promise<Nullable<Control>[]>;

    abstract control(id: number): Nullable<Control> | Promise<Nullable<Control>>;

    abstract devices(): Nullable<Device>[] | Promise<Nullable<Device>[]>;

    abstract device(id: number): Nullable<Device> | Promise<Nullable<Device>>;

    abstract events(): Nullable<Event>[] | Promise<Nullable<Event>[]>;

    abstract event(id: number): Nullable<Event> | Promise<Nullable<Event>>;

    abstract eventsummaries(): Nullable<Eventsummary>[] | Promise<Nullable<Eventsummary>[]>;

    abstract eventsummary(id: number): Nullable<Eventsummary> | Promise<Nullable<Eventsummary>>;

    abstract filters(): Nullable<Filter>[] | Promise<Nullable<Filter>[]>;

    abstract filter(id: number): Nullable<Filter> | Promise<Nullable<Filter>>;

    abstract frames(): Nullable<Frame>[] | Promise<Nullable<Frame>[]>;

    abstract frame(id: number): Nullable<Frame> | Promise<Nullable<Frame>>;

    abstract groups(): Nullable<Group>[] | Promise<Nullable<Group>[]>;

    abstract group(id: number): Nullable<Group> | Promise<Nullable<Group>>;

    abstract logs(): Nullable<Log>[] | Promise<Nullable<Log>[]>;

    abstract log(id: number): Nullable<Log> | Promise<Nullable<Log>>;

    abstract manufacturers(): Nullable<Manufacturer>[] | Promise<Nullable<Manufacturer>[]>;

    abstract manufacturer(id: number): Nullable<Manufacturer> | Promise<Nullable<Manufacturer>>;

    abstract models(): Nullable<Model>[] | Promise<Nullable<Model>[]>;

    abstract model(id: number): Nullable<Model> | Promise<Nullable<Model>>;

    abstract monitorpresets(): Nullable<Monitorpreset>[] | Promise<Nullable<Monitorpreset>[]>;

    abstract monitorpreset(id: number): Nullable<Monitorpreset> | Promise<Nullable<Monitorpreset>>;

    abstract monitors(): Nullable<Monitor>[] | Promise<Nullable<Monitor>[]>;

    abstract monitor(id: number): Nullable<Monitor> | Promise<Nullable<Monitor>>;

    abstract monitorsstatus(): Nullable<Monitorstatus>[] | Promise<Nullable<Monitorstatus>[]>;

    abstract monitorstatus(id: number): Nullable<Monitorstatus> | Promise<Nullable<Monitorstatus>>;

    abstract montagelayouts(): Nullable<Montagelayout>[] | Promise<Nullable<Montagelayout>[]>;

    abstract montagelayout(id: number): Nullable<Montagelayout> | Promise<Nullable<Montagelayout>>;

    abstract servers(): Nullable<Server>[] | Promise<Nullable<Server>[]>;

    abstract server(id: number): Nullable<Server> | Promise<Nullable<Server>>;

    abstract states(): Nullable<State>[] | Promise<Nullable<State>[]>;

    abstract state(id: number): Nullable<State> | Promise<Nullable<State>>;

    abstract storages(): Nullable<Storage>[] | Promise<Nullable<Storage>[]>;

    abstract storage(id: number): Nullable<Storage> | Promise<Nullable<Storage>>;

    abstract users(): Nullable<User>[] | Promise<Nullable<User>[]>;

    abstract user(id: number): Nullable<User> | Promise<Nullable<User>>;

    abstract zonepresets(): Nullable<Zonepreset>[] | Promise<Nullable<Zonepreset>[]>;

    abstract zonepreset(id: number): Nullable<Zonepreset> | Promise<Nullable<Zonepreset>>;

    abstract zones(): Nullable<Zone>[] | Promise<Nullable<Zone>[]>;

    abstract zone(id: number): Nullable<Zone> | Promise<Nullable<Zone>>;
}

export abstract class IMutation {
    abstract createConfig(createConfigInput: CreateConfigInput): Config | Promise<Config>;

    abstract updateConfig(updateConfigInput: UpdateConfigInput): Config | Promise<Config>;

    abstract removeConfig(Name: string): Nullable<Config> | Promise<Nullable<Config>>;

    abstract createControlpreset(createControlpresetInput: CreateControlpresetInput): Controlpreset | Promise<Controlpreset>;

    abstract updateControlpreset(updateControlpresetInput: UpdateControlpresetInput): Controlpreset | Promise<Controlpreset>;

    abstract removeControlpreset(id: number): Nullable<Controlpreset> | Promise<Nullable<Controlpreset>>;

    abstract createControl(createControlInput: CreateControlInput): Control | Promise<Control>;

    abstract updateControl(updateControlInput: UpdateControlInput): Control | Promise<Control>;

    abstract removeControl(id: number): Nullable<Control> | Promise<Nullable<Control>>;

    abstract createDevice(createDeviceInput: CreateDeviceInput): Device | Promise<Device>;

    abstract updateDevice(updateDeviceInput: UpdateDeviceInput): Device | Promise<Device>;

    abstract removeDevice(id: number): Nullable<Device> | Promise<Nullable<Device>>;

    abstract createEvent(createEventInput: CreateEventInput): Event | Promise<Event>;

    abstract updateEvent(updateEventInput: UpdateEventInput): Event | Promise<Event>;

    abstract removeEvent(id: number): Nullable<Event> | Promise<Nullable<Event>>;

    abstract createEventsummary(createEventsummaryInput: CreateEventsummaryInput): Eventsummary | Promise<Eventsummary>;

    abstract updateEventsummary(updateEventsummaryInput: UpdateEventsummaryInput): Eventsummary | Promise<Eventsummary>;

    abstract removeEventsummary(id: number): Nullable<Eventsummary> | Promise<Nullable<Eventsummary>>;

    abstract createFilter(createFilterInput: CreateFilterInput): Filter | Promise<Filter>;

    abstract updateFilter(updateFilterInput: UpdateFilterInput): Filter | Promise<Filter>;

    abstract removeFilter(id: number): Nullable<Filter> | Promise<Nullable<Filter>>;

    abstract createFrame(createFrameInput: CreateFrameInput): Frame | Promise<Frame>;

    abstract updateFrame(updateFrameInput: UpdateFrameInput): Frame | Promise<Frame>;

    abstract removeFrame(id: number): Nullable<Frame> | Promise<Nullable<Frame>>;

    abstract createGroup(createGroupInput: CreateGroupInput): Group | Promise<Group>;

    abstract updateGroup(updateGroupInput: UpdateGroupInput): Group | Promise<Group>;

    abstract removeGroup(id: number): Nullable<Group> | Promise<Nullable<Group>>;

    abstract createLog(createLogInput: CreateLogInput): Log | Promise<Log>;

    abstract updateLog(updateLogInput: UpdateLogInput): Log | Promise<Log>;

    abstract removeLog(id: number): Nullable<Log> | Promise<Nullable<Log>>;

    abstract createManufacturer(createManufacturerInput: CreateManufacturerInput): Manufacturer | Promise<Manufacturer>;

    abstract updateManufacturer(updateManufacturerInput: UpdateManufacturerInput): Manufacturer | Promise<Manufacturer>;

    abstract removeManufacturer(id: number): Nullable<Manufacturer> | Promise<Nullable<Manufacturer>>;

    abstract createModel(createModelInput: CreateModelInput): Model | Promise<Model>;

    abstract updateModel(updateModelInput: UpdateModelInput): Model | Promise<Model>;

    abstract removeModel(id: number): Nullable<Model> | Promise<Nullable<Model>>;

    abstract createMonitorpreset(createMonitorpresetInput: CreateMonitorpresetInput): Monitorpreset | Promise<Monitorpreset>;

    abstract updateMonitorpreset(updateMonitorpresetInput: UpdateMonitorpresetInput): Monitorpreset | Promise<Monitorpreset>;

    abstract removeMonitorpreset(id: number): Nullable<Monitorpreset> | Promise<Nullable<Monitorpreset>>;

    abstract createMonitor(createMonitorInput: CreateMonitorInput): Monitor | Promise<Monitor>;

    abstract updateMonitor(updateMonitorInput: UpdateMonitorInput): Monitor | Promise<Monitor>;

    abstract removeMonitor(id: number): Nullable<Monitor> | Promise<Nullable<Monitor>>;

    abstract createMonitorstatus(createMonitorstatusInput: CreateMonitorstatusInput): Monitorstatus | Promise<Monitorstatus>;

    abstract updateMonitorstatus(updateMonitorstatusInput: UpdateMonitorstatusInput): Monitorstatus | Promise<Monitorstatus>;

    abstract removeMonitorstatus(id: number): Nullable<Monitorstatus> | Promise<Nullable<Monitorstatus>>;

    abstract createMontagelayout(createMontagelayoutInput: CreateMontagelayoutInput): Montagelayout | Promise<Montagelayout>;

    abstract updateMontagelayout(updateMontagelayoutInput: UpdateMontagelayoutInput): Montagelayout | Promise<Montagelayout>;

    abstract removeMontagelayout(id: number): Nullable<Montagelayout> | Promise<Nullable<Montagelayout>>;

    abstract createServer(createServerInput: CreateServerInput): Server | Promise<Server>;

    abstract updateServer(updateServerInput: UpdateServerInput): Server | Promise<Server>;

    abstract removeServer(id: number): Nullable<Server> | Promise<Nullable<Server>>;

    abstract createState(createStateInput: CreateStateInput): State | Promise<State>;

    abstract updateState(updateStateInput: UpdateStateInput): State | Promise<State>;

    abstract removeState(id: number): Nullable<State> | Promise<Nullable<State>>;

    abstract createStorage(createStorageInput: CreateStorageInput): Storage | Promise<Storage>;

    abstract updateStorage(updateStorageInput: UpdateStorageInput): Storage | Promise<Storage>;

    abstract removeStorage(id: number): Nullable<Storage> | Promise<Nullable<Storage>>;

    abstract createUser(createUserInput: CreateUserInput): User | Promise<User>;

    abstract updateUser(updateUserInput: UpdateUserInput): User | Promise<User>;

    abstract removeUser(id: number): Nullable<User> | Promise<Nullable<User>>;

    abstract createZonepreset(createZonepresetInput: CreateZonepresetInput): Zonepreset | Promise<Zonepreset>;

    abstract updateZonepreset(updateZonepresetInput: UpdateZonepresetInput): Zonepreset | Promise<Zonepreset>;

    abstract removeZonepreset(id: number): Nullable<Zonepreset> | Promise<Nullable<Zonepreset>>;

    abstract createZone(createZoneInput: CreateZoneInput): Zone | Promise<Zone>;

    abstract updateZone(updateZoneInput: UpdateZoneInput): Zone | Promise<Zone>;

    abstract removeZone(id: number): Nullable<Zone> | Promise<Nullable<Zone>>;
}

export class Controlpreset {
    MonitorId: number;
    Preset: number;
    Label?: Nullable<string>;
}

export class Control {
    Id?: Nullable<boolean>;
    Name?: Nullable<boolean>;
    Type?: Nullable<boolean>;
    Protocol?: Nullable<boolean>;
    CanWake?: Nullable<boolean>;
    CanSleep?: Nullable<boolean>;
    CanReset?: Nullable<boolean>;
    CanReboot?: Nullable<boolean>;
    CanZoom?: Nullable<boolean>;
    CanAutoZoom?: Nullable<boolean>;
    CanZoomAbs?: Nullable<boolean>;
    CanZoomRel?: Nullable<boolean>;
    CanZoomCon?: Nullable<boolean>;
    MinZoomRange?: Nullable<boolean>;
    MaxZoomRange?: Nullable<boolean>;
    MinZoomStep?: Nullable<boolean>;
    MaxZoomStep?: Nullable<boolean>;
    HasZoomSpeed?: Nullable<boolean>;
    MinZoomSpeed?: Nullable<boolean>;
    MaxZoomSpeed?: Nullable<boolean>;
    CanFocus?: Nullable<boolean>;
    CanAutoFocus?: Nullable<boolean>;
    CanFocusAbs?: Nullable<boolean>;
    CanFocusRel?: Nullable<boolean>;
    CanFocusCon?: Nullable<boolean>;
    MinFocusRange?: Nullable<boolean>;
    MaxFocusRange?: Nullable<boolean>;
    MinFocusStep?: Nullable<boolean>;
    MaxFocusStep?: Nullable<boolean>;
    HasFocusSpeed?: Nullable<boolean>;
    MinFocusSpeed?: Nullable<boolean>;
    MaxFocusSpeed?: Nullable<boolean>;
    CanIris?: Nullable<boolean>;
    CanAutoIris?: Nullable<boolean>;
    CanIrisAbs?: Nullable<boolean>;
    CanIrisRel?: Nullable<boolean>;
    CanIrisCon?: Nullable<boolean>;
    MinIrisRange?: Nullable<boolean>;
    MaxIrisRange?: Nullable<boolean>;
    MinIrisStep?: Nullable<boolean>;
    MaxIrisStep?: Nullable<boolean>;
    HasIrisSpeed?: Nullable<boolean>;
    MinIrisSpeed?: Nullable<boolean>;
    MaxIrisSpeed?: Nullable<boolean>;
    CanGain?: Nullable<boolean>;
    CanAutoGain?: Nullable<boolean>;
    CanGainAbs?: Nullable<boolean>;
    CanGainRel?: Nullable<boolean>;
    CanGainCon?: Nullable<boolean>;
    MinGainRange?: Nullable<boolean>;
    MaxGainRange?: Nullable<boolean>;
    MinGainStep?: Nullable<boolean>;
    MaxGainStep?: Nullable<boolean>;
    HasGainSpeed?: Nullable<boolean>;
    MinGainSpeed?: Nullable<boolean>;
    MaxGainSpeed?: Nullable<boolean>;
    CanWhite?: Nullable<boolean>;
    CanAutoWhite?: Nullable<boolean>;
    CanWhiteAbs?: Nullable<boolean>;
    CanWhiteRel?: Nullable<boolean>;
    CanWhiteCon?: Nullable<boolean>;
    MinWhiteRange?: Nullable<boolean>;
    MaxWhiteRange?: Nullable<boolean>;
    MinWhiteStep?: Nullable<boolean>;
    MaxWhiteStep?: Nullable<boolean>;
    HasWhiteSpeed?: Nullable<boolean>;
    MinWhiteSpeed?: Nullable<boolean>;
    MaxWhiteSpeed?: Nullable<boolean>;
    HasPresets?: Nullable<boolean>;
    NumPresets?: Nullable<boolean>;
    HasHomePreset?: Nullable<boolean>;
    CanSetPresets?: Nullable<boolean>;
    CanMove?: Nullable<boolean>;
    CanMoveDiag?: Nullable<boolean>;
    CanMoveMap?: Nullable<boolean>;
    CanMoveAbs?: Nullable<boolean>;
    CanMoveRel?: Nullable<boolean>;
    CanMoveCon?: Nullable<boolean>;
    CanPan?: Nullable<boolean>;
    MinPanRange?: Nullable<boolean>;
    MaxPanRange?: Nullable<boolean>;
    MinPanStep?: Nullable<boolean>;
    MaxPanStep?: Nullable<boolean>;
    HasPanSpeed?: Nullable<boolean>;
    MinPanSpeed?: Nullable<boolean>;
    MaxPanSpeed?: Nullable<boolean>;
    HasTurboPan?: Nullable<boolean>;
    TurboPanSpeed?: Nullable<boolean>;
    CanTilt?: Nullable<boolean>;
    MinTiltRange?: Nullable<boolean>;
    MaxTiltRange?: Nullable<boolean>;
    MinTiltStep?: Nullable<boolean>;
    MaxTiltStep?: Nullable<boolean>;
    HasTiltSpeed?: Nullable<boolean>;
    MinTiltSpeed?: Nullable<boolean>;
    MaxTiltSpeed?: Nullable<boolean>;
    HasTurboTilt?: Nullable<boolean>;
    TurboTiltSpeed?: Nullable<boolean>;
    CanAutoScan?: Nullable<boolean>;
    NumScanPaths?: Nullable<boolean>;
}

export class Device {
    exampleField?: Nullable<number>;
}

export class Event {
    exampleField?: Nullable<number>;
}

export class Eventsummary {
    exampleField?: Nullable<number>;
}

export class Filter {
    exampleField?: Nullable<number>;
}

export class Frame {
    exampleField?: Nullable<number>;
}

export class Group {
    exampleField?: Nullable<number>;
}

export class Log {
    exampleField?: Nullable<number>;
}

export class Manufacturer {
    exampleField?: Nullable<number>;
}

export class Model {
    exampleField?: Nullable<number>;
}

export class Monitorpreset {
    exampleField?: Nullable<number>;
}

export class Monitor {
    Id: number;
    Name: string;
    Notes?: Nullable<string>;
    ServerId?: Nullable<number>;
    StorageId: number;
    Type: Monitor_Type;
    Function: Monitors_Function;
    Enabled: number;
    DecodingEnabled: number;
    LinkedMonitors?: Nullable<string>;
    Triggers: string;
    ONVIF_URL: string;
    ONVIF_Username: string;
    ONVIF_Password: string;
    ONVIF_Options: string;
    Device: string;
    Channel: number;
    Format: number;
    V4LMultiBuffer?: Nullable<number>;
    V4LCapturesPerFrame?: Nullable<number>;
    Protocol?: Nullable<string>;
    Method?: Nullable<string>;
    Host?: Nullable<string>;
    Port: string;
    SubPath: string;
    Path?: Nullable<string>;
    SecondPath?: Nullable<string>;
    Options?: Nullable<string>;
    User?: Nullable<string>;
    Pass?: Nullable<string>;
    Width: number;
    Height: number;
    Colours: number;
    Palette: number;
    Orientation?: Nullable<Monitors_Orientation>;
    Deinterlacing: number;
    DecoderHWAccelName?: Nullable<string>;
    DecoderHWAccelDevice?: Nullable<string>;
    SaveJPEGs: number;
    VideoWriter: number;
    OutputCodec?: Nullable<number>;
    Encoder?: Nullable<string>;
    OutputContainer?: Nullable<Monitors_OutputContainer>;
    EncoderParameters?: Nullable<string>;
    RecordAudio: number;
    RTSPDescribe?: Nullable<number>;
    Brightness: number;
    Contrast: number;
    Hue: number;
    Colour: number;
    EventPrefix: string;
    LabelFormat?: Nullable<string>;
    LabelX: number;
    LabelY: number;
    LabelSize: number;
    ImageBufferCount: number;
    MaxImageBufferCount: number;
    WarmupCount: number;
    PreEventCount: number;
    PostEventCount: number;
    StreamReplayBuffer: number;
    AlarmFrameCount: number;
    SectionLength: number;
    MinSectionLength: number;
    FrameSkip: number;
    MotionFrameSkip: number;
    AnalysisFPSLimit?: Nullable<number>;
    AnalysisUpdateDelay: number;
    MaxFPS?: Nullable<number>;
    AlarmMaxFPS?: Nullable<number>;
    FPSReportInterval: number;
    RefBlendPerc: number;
    AlarmRefBlendPerc: number;
    Controllable: number;
    ControlId?: Nullable<number>;
    ControlDevice?: Nullable<string>;
    ControlAddress?: Nullable<string>;
    AutoStopTimeout?: Nullable<number>;
    TrackMotion: number;
    TrackDelay?: Nullable<number>;
    ReturnLocation: number;
    ReturnDelay?: Nullable<number>;
    ModectDuringPTZ: number;
    DefaultRate: number;
    DefaultScale: number;
    DefaultCodec?: Nullable<Monitors_DefaultCodec>;
    SignalCheckPoints: number;
    SignalCheckColour: string;
    WebColour: string;
    Exif: number;
    Sequence?: Nullable<number>;
    TotalEvents?: Nullable<number>;
    ZoneCount: number;
    TotalEventDiskSpace?: Nullable<BigInt>;
    Refresh?: Nullable<number>;
    Latitude?: Nullable<number>;
    Longitude?: Nullable<number>;
    RTSPServer: boolean;
    RTSPStreamName: string;
    Importance?: Nullable<Monitors_Importance>;
    CreatedAt?: Nullable<DateTime>;
    UpdatedAt?: Nullable<DateTime>;
    LastChangeUser?: Nullable<string>;
}

export class Monitorstatus {
    exampleField?: Nullable<number>;
}

export class Montagelayout {
    exampleField?: Nullable<number>;
}

export class Server {
    exampleField?: Nullable<number>;
}

export class State {
    exampleField?: Nullable<number>;
}

export class Storage {
    exampleField?: Nullable<number>;
}

export class User {
    Id?: Nullable<number>;
    Username?: Nullable<string>;
    Language?: Nullable<string>;
    Enabled?: Nullable<boolean>;
    MaxBandwidth?: Nullable<string>;
    TokenMinExpiry?: Nullable<BigInt>;
    APIEnabled?: Nullable<boolean>;
    HomeView?: Nullable<string>;
}

export class Zonepreset {
    exampleField?: Nullable<number>;
}

export class Zone {
    exampleField?: Nullable<number>;
}

export type DateTime = any;
export type BigInt = any;
type Nullable<T> = T | null;
