import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { EnumMonitors_TypeFilter } from '../prisma/enum-monitors-type-filter.input';
import { EnumMonitors_FunctionFilter } from '../prisma/enum-monitors-function-filter.input';
import { EnumMonitors_OrientationFilter } from '../prisma/enum-monitors-orientation-filter.input';
import { EnumMonitors_OutputContainerNullableFilter } from '../prisma/enum-monitors-output-container-nullable-filter.input';
import { DecimalNullableFilter } from '../prisma/decimal-nullable-filter.input';
import { EnumMonitors_DefaultCodecFilter } from '../prisma/enum-monitors-default-codec-filter.input';
import { BigIntNullableFilter } from '../prisma/big-int-nullable-filter.input';
import { BoolFilter } from '../prisma/bool-filter.input';
import { EnumMonitors_ImportanceNullableFilter } from '../prisma/enum-monitors-importance-nullable-filter.input';

@InputType()
export class MonitorsWhereInput {

    @Field(() => [MonitorsWhereInput], {nullable:true})
    AND?: Array<MonitorsWhereInput>;

    @Field(() => [MonitorsWhereInput], {nullable:true})
    OR?: Array<MonitorsWhereInput>;

    @Field(() => [MonitorsWhereInput], {nullable:true})
    NOT?: Array<MonitorsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Notes?: StringNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    ServerId?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    StorageId?: IntFilter;

    @Field(() => EnumMonitors_TypeFilter, {nullable:true})
    Type?: EnumMonitors_TypeFilter;

    @Field(() => EnumMonitors_FunctionFilter, {nullable:true})
    Function?: EnumMonitors_FunctionFilter;

    @Field(() => IntFilter, {nullable:true})
    Enabled?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    DecodingEnabled?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    LinkedMonitors?: StringNullableFilter;

    @Field(() => StringFilter, {nullable:true})
    Triggers?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    ONVIF_URL?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    ONVIF_Username?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    ONVIF_Password?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    ONVIF_Options?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    Device?: StringFilter;

    @Field(() => IntFilter, {nullable:true})
    Channel?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Format?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    V4LMultiBuffer?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    V4LCapturesPerFrame?: IntNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Protocol?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Method?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Host?: StringNullableFilter;

    @Field(() => StringFilter, {nullable:true})
    Port?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    SubPath?: StringFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Path?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    SecondPath?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Options?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    User?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Pass?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    Width?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Height?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Colours?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Palette?: IntFilter;

    @Field(() => EnumMonitors_OrientationFilter, {nullable:true})
    Orientation?: EnumMonitors_OrientationFilter;

    @Field(() => IntFilter, {nullable:true})
    Deinterlacing?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    DecoderHWAccelName?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    DecoderHWAccelDevice?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    SaveJPEGs?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    VideoWriter?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    OutputCodec?: IntNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Encoder?: StringNullableFilter;

    @Field(() => EnumMonitors_OutputContainerNullableFilter, {nullable:true})
    OutputContainer?: EnumMonitors_OutputContainerNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    EncoderParameters?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    RecordAudio?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    RTSPDescribe?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    Brightness?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Contrast?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Hue?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Colour?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    EventPrefix?: StringFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    LabelFormat?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    LabelX?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    LabelY?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    LabelSize?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    ImageBufferCount?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MaxImageBufferCount?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    WarmupCount?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    PreEventCount?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    PostEventCount?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    StreamReplayBuffer?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AlarmFrameCount?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    SectionLength?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MinSectionLength?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    FrameSkip?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MotionFrameSkip?: IntFilter;

    @Field(() => DecimalNullableFilter, {nullable:true})
    AnalysisFPSLimit?: DecimalNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    AnalysisUpdateDelay?: IntFilter;

    @Field(() => DecimalNullableFilter, {nullable:true})
    MaxFPS?: DecimalNullableFilter;

    @Field(() => DecimalNullableFilter, {nullable:true})
    AlarmMaxFPS?: DecimalNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    FPSReportInterval?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    RefBlendPerc?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AlarmRefBlendPerc?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Controllable?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    ControlId?: IntNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    ControlDevice?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    ControlAddress?: StringNullableFilter;

    @Field(() => DecimalNullableFilter, {nullable:true})
    AutoStopTimeout?: DecimalNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    TrackMotion?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    TrackDelay?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    ReturnLocation?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    ReturnDelay?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    ModectDuringPTZ?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    DefaultRate?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    DefaultScale?: IntFilter;

    @Field(() => EnumMonitors_DefaultCodecFilter, {nullable:true})
    DefaultCodec?: EnumMonitors_DefaultCodecFilter;

    @Field(() => IntFilter, {nullable:true})
    SignalCheckPoints?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    SignalCheckColour?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    WebColour?: StringFilter;

    @Field(() => IntFilter, {nullable:true})
    Exif?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    Sequence?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    TotalEvents?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    ZoneCount?: IntFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    TotalEventDiskSpace?: BigIntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    Refresh?: IntNullableFilter;

    @Field(() => DecimalNullableFilter, {nullable:true})
    Latitude?: DecimalNullableFilter;

    @Field(() => DecimalNullableFilter, {nullable:true})
    Longitude?: DecimalNullableFilter;

    @Field(() => BoolFilter, {nullable:true})
    RTSPServer?: BoolFilter;

    @Field(() => StringFilter, {nullable:true})
    RTSPStreamName?: StringFilter;

    @Field(() => EnumMonitors_ImportanceNullableFilter, {nullable:true})
    Importance?: EnumMonitors_ImportanceNullableFilter;
}
