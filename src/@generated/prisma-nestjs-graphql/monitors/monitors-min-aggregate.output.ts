import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Monitors_Type } from '../prisma/monitors-type.enum';
import { Monitors_Function } from '../prisma/monitors-function.enum';
import { Monitors_Orientation } from './monitors-orientation.enum';
import { Monitors_OutputContainer } from '../prisma/monitors-output-container.enum';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Monitors_DefaultCodec } from './monitors-default-codec.enum';
import { Monitors_Importance } from './monitors-importance.enum';

@ObjectType()
export class MonitorsMinAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => String, {nullable:true})
    Notes?: string;

    @Field(() => Int, {nullable:true})
    ServerId?: number;

    @Field(() => Int, {nullable:true})
    StorageId?: number;

    @Field(() => Monitors_Type, {nullable:true})
    Type?: keyof typeof Monitors_Type;

    @Field(() => Monitors_Function, {nullable:true})
    Function?: keyof typeof Monitors_Function;

    @Field(() => Int, {nullable:true})
    Enabled?: number;

    @Field(() => Int, {nullable:true})
    DecodingEnabled?: number;

    @Field(() => String, {nullable:true})
    LinkedMonitors?: string;

    @Field(() => String, {nullable:true})
    Triggers?: string;

    @Field(() => String, {nullable:true})
    ONVIF_URL?: string;

    @Field(() => String, {nullable:true})
    ONVIF_Username?: string;

    @Field(() => String, {nullable:true})
    ONVIF_Password?: string;

    @Field(() => String, {nullable:true})
    ONVIF_Options?: string;

    @Field(() => String, {nullable:true})
    Device?: string;

    @Field(() => Int, {nullable:true})
    Channel?: number;

    @Field(() => Int, {nullable:true})
    Format?: number;

    @Field(() => Int, {nullable:true})
    V4LMultiBuffer?: number;

    @Field(() => Int, {nullable:true})
    V4LCapturesPerFrame?: number;

    @Field(() => String, {nullable:true})
    Protocol?: string;

    @Field(() => String, {nullable:true})
    Method?: string;

    @Field(() => String, {nullable:true})
    Host?: string;

    @Field(() => String, {nullable:true})
    Port?: string;

    @Field(() => String, {nullable:true})
    SubPath?: string;

    @Field(() => String, {nullable:true})
    Path?: string;

    @Field(() => String, {nullable:true})
    SecondPath?: string;

    @Field(() => String, {nullable:true})
    Options?: string;

    @Field(() => String, {nullable:true})
    User?: string;

    @Field(() => String, {nullable:true})
    Pass?: string;

    @Field(() => Int, {nullable:true})
    Width?: number;

    @Field(() => Int, {nullable:true})
    Height?: number;

    @Field(() => Int, {nullable:true})
    Colours?: number;

    @Field(() => Int, {nullable:true})
    Palette?: number;

    @Field(() => Monitors_Orientation, {nullable:true})
    Orientation?: keyof typeof Monitors_Orientation;

    @Field(() => Int, {nullable:true})
    Deinterlacing?: number;

    @Field(() => String, {nullable:true})
    DecoderHWAccelName?: string;

    @Field(() => String, {nullable:true})
    DecoderHWAccelDevice?: string;

    @Field(() => Int, {nullable:true})
    SaveJPEGs?: number;

    @Field(() => Int, {nullable:true})
    VideoWriter?: number;

    @Field(() => Int, {nullable:true})
    OutputCodec?: number;

    @Field(() => String, {nullable:true})
    Encoder?: string;

    @Field(() => Monitors_OutputContainer, {nullable:true})
    OutputContainer?: keyof typeof Monitors_OutputContainer;

    @Field(() => String, {nullable:true})
    EncoderParameters?: string;

    @Field(() => Int, {nullable:true})
    RecordAudio?: number;

    @Field(() => Int, {nullable:true})
    RTSPDescribe?: number;

    @Field(() => Int, {nullable:true})
    Brightness?: number;

    @Field(() => Int, {nullable:true})
    Contrast?: number;

    @Field(() => Int, {nullable:true})
    Hue?: number;

    @Field(() => Int, {nullable:true})
    Colour?: number;

    @Field(() => String, {nullable:true})
    EventPrefix?: string;

    @Field(() => String, {nullable:true})
    LabelFormat?: string;

    @Field(() => Int, {nullable:true})
    LabelX?: number;

    @Field(() => Int, {nullable:true})
    LabelY?: number;

    @Field(() => Int, {nullable:true})
    LabelSize?: number;

    @Field(() => Int, {nullable:true})
    ImageBufferCount?: number;

    @Field(() => Int, {nullable:true})
    MaxImageBufferCount?: number;

    @Field(() => Int, {nullable:true})
    WarmupCount?: number;

    @Field(() => Int, {nullable:true})
    PreEventCount?: number;

    @Field(() => Int, {nullable:true})
    PostEventCount?: number;

    @Field(() => Int, {nullable:true})
    StreamReplayBuffer?: number;

    @Field(() => Int, {nullable:true})
    AlarmFrameCount?: number;

    @Field(() => Int, {nullable:true})
    SectionLength?: number;

    @Field(() => Int, {nullable:true})
    MinSectionLength?: number;

    @Field(() => Int, {nullable:true})
    FrameSkip?: number;

    @Field(() => Int, {nullable:true})
    MotionFrameSkip?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    AnalysisFPSLimit?: any;

    @Field(() => Int, {nullable:true})
    AnalysisUpdateDelay?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS?: any;

    @Field(() => GraphQLDecimal, {nullable:true})
    AlarmMaxFPS?: any;

    @Field(() => Int, {nullable:true})
    FPSReportInterval?: number;

    @Field(() => Int, {nullable:true})
    RefBlendPerc?: number;

    @Field(() => Int, {nullable:true})
    AlarmRefBlendPerc?: number;

    @Field(() => Int, {nullable:true})
    Controllable?: number;

    @Field(() => Int, {nullable:true})
    ControlId?: number;

    @Field(() => String, {nullable:true})
    ControlDevice?: string;

    @Field(() => String, {nullable:true})
    ControlAddress?: string;

    @Field(() => GraphQLDecimal, {nullable:true})
    AutoStopTimeout?: any;

    @Field(() => Int, {nullable:true})
    TrackMotion?: number;

    @Field(() => Int, {nullable:true})
    TrackDelay?: number;

    @Field(() => Int, {nullable:true})
    ReturnLocation?: number;

    @Field(() => Int, {nullable:true})
    ReturnDelay?: number;

    @Field(() => Int, {nullable:true})
    ModectDuringPTZ?: number;

    @Field(() => Int, {nullable:true})
    DefaultRate?: number;

    @Field(() => Int, {nullable:true})
    DefaultScale?: number;

    @Field(() => Monitors_DefaultCodec, {nullable:true})
    DefaultCodec?: keyof typeof Monitors_DefaultCodec;

    @Field(() => Int, {nullable:true})
    SignalCheckPoints?: number;

    @Field(() => String, {nullable:true})
    SignalCheckColour?: string;

    @Field(() => String, {nullable:true})
    WebColour?: string;

    @Field(() => Int, {nullable:true})
    Exif?: number;

    @Field(() => Int, {nullable:true})
    Sequence?: number;

    @Field(() => Int, {nullable:true})
    TotalEvents?: number;

    @Field(() => Int, {nullable:true})
    ZoneCount?: number;

    @Field(() => String, {nullable:true})
    TotalEventDiskSpace?: bigint | number;

    @Field(() => Int, {nullable:true})
    Refresh?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    Latitude?: any;

    @Field(() => GraphQLDecimal, {nullable:true})
    Longitude?: any;

    @Field(() => Boolean, {nullable:true})
    RTSPServer?: boolean;

    @Field(() => String, {nullable:true})
    RTSPStreamName?: string;

    @Field(() => Monitors_Importance, {nullable:true})
    Importance?: keyof typeof Monitors_Importance;
}
