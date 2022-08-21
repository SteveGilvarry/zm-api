import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class MonitorsAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    ServerId?: number;

    @Field(() => Float, {nullable:true})
    StorageId?: number;

    @Field(() => Float, {nullable:true})
    Enabled?: number;

    @Field(() => Float, {nullable:true})
    DecodingEnabled?: number;

    @Field(() => Float, {nullable:true})
    Channel?: number;

    @Field(() => Float, {nullable:true})
    Format?: number;

    @Field(() => Float, {nullable:true})
    V4LMultiBuffer?: number;

    @Field(() => Float, {nullable:true})
    V4LCapturesPerFrame?: number;

    @Field(() => Float, {nullable:true})
    Width?: number;

    @Field(() => Float, {nullable:true})
    Height?: number;

    @Field(() => Float, {nullable:true})
    Colours?: number;

    @Field(() => Float, {nullable:true})
    Palette?: number;

    @Field(() => Float, {nullable:true})
    Deinterlacing?: number;

    @Field(() => Float, {nullable:true})
    SaveJPEGs?: number;

    @Field(() => Float, {nullable:true})
    VideoWriter?: number;

    @Field(() => Float, {nullable:true})
    OutputCodec?: number;

    @Field(() => Float, {nullable:true})
    RecordAudio?: number;

    @Field(() => Float, {nullable:true})
    RTSPDescribe?: number;

    @Field(() => Float, {nullable:true})
    Brightness?: number;

    @Field(() => Float, {nullable:true})
    Contrast?: number;

    @Field(() => Float, {nullable:true})
    Hue?: number;

    @Field(() => Float, {nullable:true})
    Colour?: number;

    @Field(() => Float, {nullable:true})
    LabelX?: number;

    @Field(() => Float, {nullable:true})
    LabelY?: number;

    @Field(() => Float, {nullable:true})
    LabelSize?: number;

    @Field(() => Float, {nullable:true})
    ImageBufferCount?: number;

    @Field(() => Float, {nullable:true})
    MaxImageBufferCount?: number;

    @Field(() => Float, {nullable:true})
    WarmupCount?: number;

    @Field(() => Float, {nullable:true})
    PreEventCount?: number;

    @Field(() => Float, {nullable:true})
    PostEventCount?: number;

    @Field(() => Float, {nullable:true})
    StreamReplayBuffer?: number;

    @Field(() => Float, {nullable:true})
    AlarmFrameCount?: number;

    @Field(() => Float, {nullable:true})
    SectionLength?: number;

    @Field(() => Float, {nullable:true})
    MinSectionLength?: number;

    @Field(() => Float, {nullable:true})
    FrameSkip?: number;

    @Field(() => Float, {nullable:true})
    MotionFrameSkip?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    AnalysisFPSLimit?: Decimal;

    @Field(() => Float, {nullable:true})
    AnalysisUpdateDelay?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    AlarmMaxFPS?: Decimal;

    @Field(() => Float, {nullable:true})
    FPSReportInterval?: number;

    @Field(() => Float, {nullable:true})
    RefBlendPerc?: number;

    @Field(() => Float, {nullable:true})
    AlarmRefBlendPerc?: number;

    @Field(() => Float, {nullable:true})
    Controllable?: number;

    @Field(() => Float, {nullable:true})
    ControlId?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    AutoStopTimeout?: Decimal;

    @Field(() => Float, {nullable:true})
    TrackMotion?: number;

    @Field(() => Float, {nullable:true})
    TrackDelay?: number;

    @Field(() => Float, {nullable:true})
    ReturnLocation?: number;

    @Field(() => Float, {nullable:true})
    ReturnDelay?: number;

    @Field(() => Float, {nullable:true})
    ModectDuringPTZ?: number;

    @Field(() => Float, {nullable:true})
    DefaultRate?: number;

    @Field(() => Float, {nullable:true})
    DefaultScale?: number;

    @Field(() => Float, {nullable:true})
    SignalCheckPoints?: number;

    @Field(() => Float, {nullable:true})
    Exif?: number;

    @Field(() => Float, {nullable:true})
    Sequence?: number;

    @Field(() => Float, {nullable:true})
    TotalEvents?: number;

    @Field(() => Float, {nullable:true})
    ZoneCount?: number;

    @Field(() => Float, {nullable:true})
    TotalEventDiskSpace?: number;

    @Field(() => Float, {nullable:true})
    Refresh?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    Latitude?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    Longitude?: Decimal;
}
