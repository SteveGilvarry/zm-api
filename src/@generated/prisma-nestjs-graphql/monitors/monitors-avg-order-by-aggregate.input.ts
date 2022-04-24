import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class MonitorsAvgOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ServerId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StorageId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Enabled?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DecodingEnabled?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Channel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Format?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    V4LMultiBuffer?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    V4LCapturesPerFrame?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Width?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Height?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Colours?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Palette?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Deinterlacing?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SaveJPEGs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    VideoWriter?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    OutputCodec?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    RecordAudio?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    RTSPDescribe?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Brightness?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Contrast?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Hue?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Colour?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    LabelX?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    LabelY?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    LabelSize?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ImageBufferCount?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxImageBufferCount?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    WarmupCount?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    PreEventCount?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    PostEventCount?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StreamReplayBuffer?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmFrameCount?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SectionLength?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinSectionLength?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FrameSkip?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MotionFrameSkip?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AnalysisFPSLimit?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AnalysisUpdateDelay?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxFPS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmMaxFPS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FPSReportInterval?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    RefBlendPerc?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmRefBlendPerc?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Controllable?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ControlId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoStopTimeout?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TrackMotion?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TrackDelay?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ReturnLocation?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ReturnDelay?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ModectDuringPTZ?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultRate?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultScale?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SignalCheckPoints?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Exif?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Sequence?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotalEvents?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ZoneCount?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotalEventDiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Refresh?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Latitude?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Longitude?: keyof typeof SortOrder;
}
