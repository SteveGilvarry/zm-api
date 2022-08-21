import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Monitors_Type } from '../prisma/monitors-type.enum';
import { Monitors_Function } from '../prisma/monitors-function.enum';
import { Monitors_Orientation } from './monitors-orientation.enum';
import { Monitors_OutputContainer } from '../prisma/monitors-output-container.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Monitors_DefaultCodec } from './monitors-default-codec.enum';
import { Monitors_Importance } from './monitors-importance.enum';
import { MonitorsCountAggregate } from './monitors-count-aggregate.output';
import { MonitorsAvgAggregate } from './monitors-avg-aggregate.output';
import { MonitorsSumAggregate } from './monitors-sum-aggregate.output';
import { MonitorsMinAggregate } from './monitors-min-aggregate.output';
import { MonitorsMaxAggregate } from './monitors-max-aggregate.output';

@ObjectType()
export class MonitorsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => String, {nullable:true})
    Notes?: string;

    @Field(() => Int, {nullable:true})
    ServerId?: number;

    @Field(() => Int, {nullable:false})
    StorageId!: number;

    @Field(() => Monitors_Type, {nullable:false})
    Type!: keyof typeof Monitors_Type;

    @Field(() => Monitors_Function, {nullable:false})
    Function!: keyof typeof Monitors_Function;

    @Field(() => Int, {nullable:false})
    Enabled!: number;

    @Field(() => Int, {nullable:false})
    DecodingEnabled!: number;

    @Field(() => String, {nullable:true})
    LinkedMonitors?: string;

    @Field(() => String, {nullable:false})
    Triggers!: string;

    @Field(() => String, {nullable:false})
    ONVIF_URL!: string;

    @Field(() => String, {nullable:false})
    ONVIF_Username!: string;

    @Field(() => String, {nullable:false})
    ONVIF_Password!: string;

    @Field(() => String, {nullable:false})
    ONVIF_Options!: string;

    @Field(() => String, {nullable:false})
    Device!: string;

    @Field(() => Int, {nullable:false})
    Channel!: number;

    @Field(() => Int, {nullable:false})
    Format!: number;

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

    @Field(() => String, {nullable:false})
    Port!: string;

    @Field(() => String, {nullable:false})
    SubPath!: string;

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

    @Field(() => Int, {nullable:false})
    Width!: number;

    @Field(() => Int, {nullable:false})
    Height!: number;

    @Field(() => Int, {nullable:false})
    Colours!: number;

    @Field(() => Int, {nullable:false})
    Palette!: number;

    @Field(() => Monitors_Orientation, {nullable:false})
    Orientation!: keyof typeof Monitors_Orientation;

    @Field(() => Int, {nullable:false})
    Deinterlacing!: number;

    @Field(() => String, {nullable:true})
    DecoderHWAccelName?: string;

    @Field(() => String, {nullable:true})
    DecoderHWAccelDevice?: string;

    @Field(() => Int, {nullable:false})
    SaveJPEGs!: number;

    @Field(() => Int, {nullable:false})
    VideoWriter!: number;

    @Field(() => Int, {nullable:true})
    OutputCodec?: number;

    @Field(() => String, {nullable:true})
    Encoder?: string;

    @Field(() => Monitors_OutputContainer, {nullable:true})
    OutputContainer?: keyof typeof Monitors_OutputContainer;

    @Field(() => String, {nullable:true})
    EncoderParameters?: string;

    @Field(() => Int, {nullable:false})
    RecordAudio!: number;

    @Field(() => Int, {nullable:true})
    RTSPDescribe?: number;

    @Field(() => Int, {nullable:false})
    Brightness!: number;

    @Field(() => Int, {nullable:false})
    Contrast!: number;

    @Field(() => Int, {nullable:false})
    Hue!: number;

    @Field(() => Int, {nullable:false})
    Colour!: number;

    @Field(() => String, {nullable:false})
    EventPrefix!: string;

    @Field(() => String, {nullable:true})
    LabelFormat?: string;

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

    @Field(() => GraphQLDecimal, {nullable:true})
    AnalysisFPSLimit?: Decimal;

    @Field(() => Int, {nullable:false})
    AnalysisUpdateDelay!: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    AlarmMaxFPS?: Decimal;

    @Field(() => Int, {nullable:false})
    FPSReportInterval!: number;

    @Field(() => Int, {nullable:false})
    RefBlendPerc!: number;

    @Field(() => Int, {nullable:false})
    AlarmRefBlendPerc!: number;

    @Field(() => Int, {nullable:false})
    Controllable!: number;

    @Field(() => Int, {nullable:true})
    ControlId?: number;

    @Field(() => String, {nullable:true})
    ControlDevice?: string;

    @Field(() => String, {nullable:true})
    ControlAddress?: string;

    @Field(() => GraphQLDecimal, {nullable:true})
    AutoStopTimeout?: Decimal;

    @Field(() => Int, {nullable:false})
    TrackMotion!: number;

    @Field(() => Int, {nullable:true})
    TrackDelay?: number;

    @Field(() => Int, {nullable:false})
    ReturnLocation!: number;

    @Field(() => Int, {nullable:true})
    ReturnDelay?: number;

    @Field(() => Int, {nullable:false})
    ModectDuringPTZ!: number;

    @Field(() => Int, {nullable:false})
    DefaultRate!: number;

    @Field(() => Int, {nullable:false})
    DefaultScale!: number;

    @Field(() => Monitors_DefaultCodec, {nullable:false})
    DefaultCodec!: keyof typeof Monitors_DefaultCodec;

    @Field(() => Int, {nullable:false})
    SignalCheckPoints!: number;

    @Field(() => String, {nullable:false})
    SignalCheckColour!: string;

    @Field(() => String, {nullable:false})
    WebColour!: string;

    @Field(() => Int, {nullable:false})
    Exif!: number;

    @Field(() => Int, {nullable:true})
    Sequence?: number;

    @Field(() => Int, {nullable:true})
    TotalEvents?: number;

    @Field(() => Int, {nullable:false})
    ZoneCount!: number;

    @Field(() => String, {nullable:true})
    TotalEventDiskSpace?: bigint | number;

    @Field(() => Int, {nullable:true})
    Refresh?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    Latitude?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    Longitude?: Decimal;

    @Field(() => Boolean, {nullable:false})
    RTSPServer!: boolean;

    @Field(() => String, {nullable:false})
    RTSPStreamName!: string;

    @Field(() => Monitors_Importance, {nullable:true})
    Importance?: keyof typeof Monitors_Importance;

    @Field(() => MonitorsCountAggregate, {nullable:true})
    _count?: MonitorsCountAggregate;

    @Field(() => MonitorsAvgAggregate, {nullable:true})
    _avg?: MonitorsAvgAggregate;

    @Field(() => MonitorsSumAggregate, {nullable:true})
    _sum?: MonitorsSumAggregate;

    @Field(() => MonitorsMinAggregate, {nullable:true})
    _min?: MonitorsMinAggregate;

    @Field(() => MonitorsMaxAggregate, {nullable:true})
    _max?: MonitorsMaxAggregate;
}
