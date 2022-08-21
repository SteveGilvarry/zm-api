import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class MonitorsSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    ServerId?: number;

    @Field(() => Int, {nullable:true})
    StorageId?: number;

    @Field(() => Int, {nullable:true})
    Enabled?: number;

    @Field(() => Int, {nullable:true})
    DecodingEnabled?: number;

    @Field(() => Int, {nullable:true})
    Channel?: number;

    @Field(() => Int, {nullable:true})
    Format?: number;

    @Field(() => Int, {nullable:true})
    V4LMultiBuffer?: number;

    @Field(() => Int, {nullable:true})
    V4LCapturesPerFrame?: number;

    @Field(() => Int, {nullable:true})
    Width?: number;

    @Field(() => Int, {nullable:true})
    Height?: number;

    @Field(() => Int, {nullable:true})
    Colours?: number;

    @Field(() => Int, {nullable:true})
    Palette?: number;

    @Field(() => Int, {nullable:true})
    Deinterlacing?: number;

    @Field(() => Int, {nullable:true})
    SaveJPEGs?: number;

    @Field(() => Int, {nullable:true})
    VideoWriter?: number;

    @Field(() => Int, {nullable:true})
    OutputCodec?: number;

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
    AnalysisFPSLimit?: Decimal;

    @Field(() => Int, {nullable:true})
    AnalysisUpdateDelay?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    AlarmMaxFPS?: Decimal;

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

    @Field(() => GraphQLDecimal, {nullable:true})
    AutoStopTimeout?: Decimal;

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

    @Field(() => Int, {nullable:true})
    SignalCheckPoints?: number;

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
    Latitude?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    Longitude?: Decimal;
}
