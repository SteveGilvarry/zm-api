import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class MonitorsCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Notes!: number;

    @Field(() => Int, {nullable:false})
    ServerId!: number;

    @Field(() => Int, {nullable:false})
    StorageId!: number;

    @Field(() => Int, {nullable:false})
    Type!: number;

    @Field(() => Int, {nullable:false})
    Function!: number;

    @Field(() => Int, {nullable:false})
    Enabled!: number;

    @Field(() => Int, {nullable:false})
    DecodingEnabled!: number;

    @Field(() => Int, {nullable:false})
    LinkedMonitors!: number;

    @Field(() => Int, {nullable:false})
    Triggers!: number;

    @Field(() => Int, {nullable:false})
    ONVIF_URL!: number;

    @Field(() => Int, {nullable:false})
    ONVIF_Username!: number;

    @Field(() => Int, {nullable:false})
    ONVIF_Password!: number;

    @Field(() => Int, {nullable:false})
    ONVIF_Options!: number;

    @Field(() => Int, {nullable:false})
    Device!: number;

    @Field(() => Int, {nullable:false})
    Channel!: number;

    @Field(() => Int, {nullable:false})
    Format!: number;

    @Field(() => Int, {nullable:false})
    V4LMultiBuffer!: number;

    @Field(() => Int, {nullable:false})
    V4LCapturesPerFrame!: number;

    @Field(() => Int, {nullable:false})
    Protocol!: number;

    @Field(() => Int, {nullable:false})
    Method!: number;

    @Field(() => Int, {nullable:false})
    Host!: number;

    @Field(() => Int, {nullable:false})
    Port!: number;

    @Field(() => Int, {nullable:false})
    SubPath!: number;

    @Field(() => Int, {nullable:false})
    Path!: number;

    @Field(() => Int, {nullable:false})
    SecondPath!: number;

    @Field(() => Int, {nullable:false})
    Options!: number;

    @Field(() => Int, {nullable:false})
    User!: number;

    @Field(() => Int, {nullable:false})
    Pass!: number;

    @Field(() => Int, {nullable:false})
    Width!: number;

    @Field(() => Int, {nullable:false})
    Height!: number;

    @Field(() => Int, {nullable:false})
    Colours!: number;

    @Field(() => Int, {nullable:false})
    Palette!: number;

    @Field(() => Int, {nullable:false})
    Orientation!: number;

    @Field(() => Int, {nullable:false})
    Deinterlacing!: number;

    @Field(() => Int, {nullable:false})
    DecoderHWAccelName!: number;

    @Field(() => Int, {nullable:false})
    DecoderHWAccelDevice!: number;

    @Field(() => Int, {nullable:false})
    SaveJPEGs!: number;

    @Field(() => Int, {nullable:false})
    VideoWriter!: number;

    @Field(() => Int, {nullable:false})
    OutputCodec!: number;

    @Field(() => Int, {nullable:false})
    Encoder!: number;

    @Field(() => Int, {nullable:false})
    OutputContainer!: number;

    @Field(() => Int, {nullable:false})
    EncoderParameters!: number;

    @Field(() => Int, {nullable:false})
    RecordAudio!: number;

    @Field(() => Int, {nullable:false})
    RTSPDescribe!: number;

    @Field(() => Int, {nullable:false})
    Brightness!: number;

    @Field(() => Int, {nullable:false})
    Contrast!: number;

    @Field(() => Int, {nullable:false})
    Hue!: number;

    @Field(() => Int, {nullable:false})
    Colour!: number;

    @Field(() => Int, {nullable:false})
    EventPrefix!: number;

    @Field(() => Int, {nullable:false})
    LabelFormat!: number;

    @Field(() => Int, {nullable:false})
    LabelX!: number;

    @Field(() => Int, {nullable:false})
    LabelY!: number;

    @Field(() => Int, {nullable:false})
    LabelSize!: number;

    @Field(() => Int, {nullable:false})
    ImageBufferCount!: number;

    @Field(() => Int, {nullable:false})
    MaxImageBufferCount!: number;

    @Field(() => Int, {nullable:false})
    WarmupCount!: number;

    @Field(() => Int, {nullable:false})
    PreEventCount!: number;

    @Field(() => Int, {nullable:false})
    PostEventCount!: number;

    @Field(() => Int, {nullable:false})
    StreamReplayBuffer!: number;

    @Field(() => Int, {nullable:false})
    AlarmFrameCount!: number;

    @Field(() => Int, {nullable:false})
    SectionLength!: number;

    @Field(() => Int, {nullable:false})
    MinSectionLength!: number;

    @Field(() => Int, {nullable:false})
    FrameSkip!: number;

    @Field(() => Int, {nullable:false})
    MotionFrameSkip!: number;

    @Field(() => Int, {nullable:false})
    AnalysisFPSLimit!: number;

    @Field(() => Int, {nullable:false})
    AnalysisUpdateDelay!: number;

    @Field(() => Int, {nullable:false})
    MaxFPS!: number;

    @Field(() => Int, {nullable:false})
    AlarmMaxFPS!: number;

    @Field(() => Int, {nullable:false})
    FPSReportInterval!: number;

    @Field(() => Int, {nullable:false})
    RefBlendPerc!: number;

    @Field(() => Int, {nullable:false})
    AlarmRefBlendPerc!: number;

    @Field(() => Int, {nullable:false})
    Controllable!: number;

    @Field(() => Int, {nullable:false})
    ControlId!: number;

    @Field(() => Int, {nullable:false})
    ControlDevice!: number;

    @Field(() => Int, {nullable:false})
    ControlAddress!: number;

    @Field(() => Int, {nullable:false})
    AutoStopTimeout!: number;

    @Field(() => Int, {nullable:false})
    TrackMotion!: number;

    @Field(() => Int, {nullable:false})
    TrackDelay!: number;

    @Field(() => Int, {nullable:false})
    ReturnLocation!: number;

    @Field(() => Int, {nullable:false})
    ReturnDelay!: number;

    @Field(() => Int, {nullable:false})
    ModectDuringPTZ!: number;

    @Field(() => Int, {nullable:false})
    DefaultRate!: number;

    @Field(() => Int, {nullable:false})
    DefaultScale!: number;

    @Field(() => Int, {nullable:false})
    DefaultCodec!: number;

    @Field(() => Int, {nullable:false})
    SignalCheckPoints!: number;

    @Field(() => Int, {nullable:false})
    SignalCheckColour!: number;

    @Field(() => Int, {nullable:false})
    WebColour!: number;

    @Field(() => Int, {nullable:false})
    Exif!: number;

    @Field(() => Int, {nullable:false})
    Sequence!: number;

    @Field(() => Int, {nullable:false})
    TotalEvents!: number;

    @Field(() => Int, {nullable:false})
    ZoneCount!: number;

    @Field(() => Int, {nullable:false})
    TotalEventDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    Refresh!: number;

    @Field(() => Int, {nullable:false})
    Latitude!: number;

    @Field(() => Int, {nullable:false})
    Longitude!: number;

    @Field(() => Int, {nullable:false})
    RTSPServer!: number;

    @Field(() => Int, {nullable:false})
    RTSPStreamName!: number;

    @Field(() => Int, {nullable:false})
    Importance!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
