import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class MonitorsCountAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Notes?: true;

    @Field(() => Boolean, {nullable:true})
    ServerId?: true;

    @Field(() => Boolean, {nullable:true})
    StorageId?: true;

    @Field(() => Boolean, {nullable:true})
    Type?: true;

    @Field(() => Boolean, {nullable:true})
    Function?: true;

    @Field(() => Boolean, {nullable:true})
    Enabled?: true;

    @Field(() => Boolean, {nullable:true})
    DecodingEnabled?: true;

    @Field(() => Boolean, {nullable:true})
    LinkedMonitors?: true;

    @Field(() => Boolean, {nullable:true})
    Triggers?: true;

    @Field(() => Boolean, {nullable:true})
    ONVIF_URL?: true;

    @Field(() => Boolean, {nullable:true})
    ONVIF_Username?: true;

    @Field(() => Boolean, {nullable:true})
    ONVIF_Password?: true;

    @Field(() => Boolean, {nullable:true})
    ONVIF_Options?: true;

    @Field(() => Boolean, {nullable:true})
    Device?: true;

    @Field(() => Boolean, {nullable:true})
    Channel?: true;

    @Field(() => Boolean, {nullable:true})
    Format?: true;

    @Field(() => Boolean, {nullable:true})
    V4LMultiBuffer?: true;

    @Field(() => Boolean, {nullable:true})
    V4LCapturesPerFrame?: true;

    @Field(() => Boolean, {nullable:true})
    Protocol?: true;

    @Field(() => Boolean, {nullable:true})
    Method?: true;

    @Field(() => Boolean, {nullable:true})
    Host?: true;

    @Field(() => Boolean, {nullable:true})
    Port?: true;

    @Field(() => Boolean, {nullable:true})
    SubPath?: true;

    @Field(() => Boolean, {nullable:true})
    Path?: true;

    @Field(() => Boolean, {nullable:true})
    SecondPath?: true;

    @Field(() => Boolean, {nullable:true})
    Options?: true;

    @Field(() => Boolean, {nullable:true})
    User?: true;

    @Field(() => Boolean, {nullable:true})
    Pass?: true;

    @Field(() => Boolean, {nullable:true})
    Width?: true;

    @Field(() => Boolean, {nullable:true})
    Height?: true;

    @Field(() => Boolean, {nullable:true})
    Colours?: true;

    @Field(() => Boolean, {nullable:true})
    Palette?: true;

    @Field(() => Boolean, {nullable:true})
    Orientation?: true;

    @Field(() => Boolean, {nullable:true})
    Deinterlacing?: true;

    @Field(() => Boolean, {nullable:true})
    DecoderHWAccelName?: true;

    @Field(() => Boolean, {nullable:true})
    DecoderHWAccelDevice?: true;

    @Field(() => Boolean, {nullable:true})
    SaveJPEGs?: true;

    @Field(() => Boolean, {nullable:true})
    VideoWriter?: true;

    @Field(() => Boolean, {nullable:true})
    OutputCodec?: true;

    @Field(() => Boolean, {nullable:true})
    Encoder?: true;

    @Field(() => Boolean, {nullable:true})
    OutputContainer?: true;

    @Field(() => Boolean, {nullable:true})
    EncoderParameters?: true;

    @Field(() => Boolean, {nullable:true})
    RecordAudio?: true;

    @Field(() => Boolean, {nullable:true})
    RTSPDescribe?: true;

    @Field(() => Boolean, {nullable:true})
    Brightness?: true;

    @Field(() => Boolean, {nullable:true})
    Contrast?: true;

    @Field(() => Boolean, {nullable:true})
    Hue?: true;

    @Field(() => Boolean, {nullable:true})
    Colour?: true;

    @Field(() => Boolean, {nullable:true})
    EventPrefix?: true;

    @Field(() => Boolean, {nullable:true})
    LabelFormat?: true;

    @Field(() => Boolean, {nullable:true})
    LabelX?: true;

    @Field(() => Boolean, {nullable:true})
    LabelY?: true;

    @Field(() => Boolean, {nullable:true})
    LabelSize?: true;

    @Field(() => Boolean, {nullable:true})
    ImageBufferCount?: true;

    @Field(() => Boolean, {nullable:true})
    MaxImageBufferCount?: true;

    @Field(() => Boolean, {nullable:true})
    WarmupCount?: true;

    @Field(() => Boolean, {nullable:true})
    PreEventCount?: true;

    @Field(() => Boolean, {nullable:true})
    PostEventCount?: true;

    @Field(() => Boolean, {nullable:true})
    StreamReplayBuffer?: true;

    @Field(() => Boolean, {nullable:true})
    AlarmFrameCount?: true;

    @Field(() => Boolean, {nullable:true})
    SectionLength?: true;

    @Field(() => Boolean, {nullable:true})
    MinSectionLength?: true;

    @Field(() => Boolean, {nullable:true})
    FrameSkip?: true;

    @Field(() => Boolean, {nullable:true})
    MotionFrameSkip?: true;

    @Field(() => Boolean, {nullable:true})
    AnalysisFPSLimit?: true;

    @Field(() => Boolean, {nullable:true})
    AnalysisUpdateDelay?: true;

    @Field(() => Boolean, {nullable:true})
    MaxFPS?: true;

    @Field(() => Boolean, {nullable:true})
    AlarmMaxFPS?: true;

    @Field(() => Boolean, {nullable:true})
    FPSReportInterval?: true;

    @Field(() => Boolean, {nullable:true})
    RefBlendPerc?: true;

    @Field(() => Boolean, {nullable:true})
    AlarmRefBlendPerc?: true;

    @Field(() => Boolean, {nullable:true})
    Controllable?: true;

    @Field(() => Boolean, {nullable:true})
    ControlId?: true;

    @Field(() => Boolean, {nullable:true})
    ControlDevice?: true;

    @Field(() => Boolean, {nullable:true})
    ControlAddress?: true;

    @Field(() => Boolean, {nullable:true})
    AutoStopTimeout?: true;

    @Field(() => Boolean, {nullable:true})
    TrackMotion?: true;

    @Field(() => Boolean, {nullable:true})
    TrackDelay?: true;

    @Field(() => Boolean, {nullable:true})
    ReturnLocation?: true;

    @Field(() => Boolean, {nullable:true})
    ReturnDelay?: true;

    @Field(() => Boolean, {nullable:true})
    ModectDuringPTZ?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultRate?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultScale?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultCodec?: true;

    @Field(() => Boolean, {nullable:true})
    SignalCheckPoints?: true;

    @Field(() => Boolean, {nullable:true})
    SignalCheckColour?: true;

    @Field(() => Boolean, {nullable:true})
    WebColour?: true;

    @Field(() => Boolean, {nullable:true})
    Exif?: true;

    @Field(() => Boolean, {nullable:true})
    Sequence?: true;

    @Field(() => Boolean, {nullable:true})
    TotalEvents?: true;

    @Field(() => Boolean, {nullable:true})
    ZoneCount?: true;

    @Field(() => Boolean, {nullable:true})
    TotalEventDiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    Refresh?: true;

    @Field(() => Boolean, {nullable:true})
    Latitude?: true;

    @Field(() => Boolean, {nullable:true})
    Longitude?: true;

    @Field(() => Boolean, {nullable:true})
    RTSPServer?: true;

    @Field(() => Boolean, {nullable:true})
    RTSPStreamName?: true;

    @Field(() => Boolean, {nullable:true})
    Importance?: true;

    @Field(() => Boolean, {nullable:true})
    _all?: true;
}
