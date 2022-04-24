import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Monitors_Type } from '../prisma/monitors-type.enum';
import { Monitors_Function } from '../prisma/monitors-function.enum';
import { Monitors_Orientation } from './monitors-orientation.enum';
import { Monitors_OutputContainer } from '../prisma/monitors-output-container.enum';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Monitors_DefaultCodec } from './monitors-default-codec.enum';
import { Monitors_Importance } from './monitors-importance.enum';

@ObjectType()
export class Monitors {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false,defaultValue:''})
    Name!: string;

    @Field(() => String, {nullable:true})
    Notes!: string | null;

    @Field(() => Int, {nullable:true})
    ServerId!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    StorageId!: number;

    @Field(() => Monitors_Type, {nullable:false,defaultValue:'Local'})
    Type!: keyof typeof Monitors_Type;

    @Field(() => Monitors_Function, {nullable:false,defaultValue:'Monitor'})
    Function!: keyof typeof Monitors_Function;

    @Field(() => Int, {nullable:false,defaultValue:1})
    Enabled!: number;

    @Field(() => Int, {nullable:false,defaultValue:1})
    DecodingEnabled!: number;

    @Field(() => String, {nullable:true})
    LinkedMonitors!: string | null;

    @Field(() => String, {nullable:false,defaultValue:''})
    Triggers!: string;

    @Field(() => String, {nullable:false,defaultValue:''})
    ONVIF_URL!: string;

    @Field(() => String, {nullable:false,defaultValue:''})
    ONVIF_Username!: string;

    @Field(() => String, {nullable:false,defaultValue:''})
    ONVIF_Password!: string;

    @Field(() => String, {nullable:false,defaultValue:''})
    ONVIF_Options!: string;

    @Field(() => String, {nullable:false})
    Device!: string;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Channel!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Format!: number;

    @Field(() => Int, {nullable:true})
    V4LMultiBuffer!: number | null;

    @Field(() => Int, {nullable:true})
    V4LCapturesPerFrame!: number | null;

    @Field(() => String, {nullable:true})
    Protocol!: string | null;

    @Field(() => String, {nullable:true})
    Method!: string | null;

    @Field(() => String, {nullable:true})
    Host!: string | null;

    @Field(() => String, {nullable:false,defaultValue:''})
    Port!: string;

    @Field(() => String, {nullable:false,defaultValue:''})
    SubPath!: string;

    @Field(() => String, {nullable:true})
    Path!: string | null;

    @Field(() => String, {nullable:true})
    SecondPath!: string | null;

    @Field(() => String, {nullable:true})
    Options!: string | null;

    @Field(() => String, {nullable:true})
    User!: string | null;

    @Field(() => String, {nullable:true})
    Pass!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Width!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Height!: number;

    @Field(() => Int, {nullable:false,defaultValue:1})
    Colours!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Palette!: number;

    @Field(() => Monitors_Orientation, {nullable:false,defaultValue:'ROTATE_0'})
    Orientation!: keyof typeof Monitors_Orientation;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Deinterlacing!: number;

    @Field(() => String, {nullable:true})
    DecoderHWAccelName!: string | null;

    @Field(() => String, {nullable:true})
    DecoderHWAccelDevice!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:3})
    SaveJPEGs!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    VideoWriter!: number;

    @Field(() => Int, {nullable:true,defaultValue:0})
    OutputCodec!: number | null;

    @Field(() => String, {nullable:true})
    Encoder!: string | null;

    @Field(() => Monitors_OutputContainer, {nullable:true,defaultValue:'auto'})
    OutputContainer!: keyof typeof Monitors_OutputContainer | null;

    @Field(() => String, {nullable:true})
    EncoderParameters!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    RecordAudio!: number;

    @Field(() => Int, {nullable:true})
    RTSPDescribe!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:-1})
    Brightness!: number;

    @Field(() => Int, {nullable:false,defaultValue:-1})
    Contrast!: number;

    @Field(() => Int, {nullable:false,defaultValue:-1})
    Hue!: number;

    @Field(() => Int, {nullable:false,defaultValue:-1})
    Colour!: number;

    @Field(() => String, {nullable:false,defaultValue:'Event-'})
    EventPrefix!: string;

    @Field(() => String, {nullable:true})
    LabelFormat!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    LabelX!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    LabelY!: number;

    @Field(() => Int, {nullable:false,defaultValue:1})
    LabelSize!: number;

    @Field(() => Int, {nullable:false,defaultValue:100})
    ImageBufferCount!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MaxImageBufferCount!: number;

    @Field(() => Int, {nullable:false,defaultValue:25})
    WarmupCount!: number;

    @Field(() => Int, {nullable:false,defaultValue:10})
    PreEventCount!: number;

    @Field(() => Int, {nullable:false,defaultValue:10})
    PostEventCount!: number;

    @Field(() => Int, {nullable:false,defaultValue:1000})
    StreamReplayBuffer!: number;

    @Field(() => Int, {nullable:false,defaultValue:1})
    AlarmFrameCount!: number;

    @Field(() => Int, {nullable:false,defaultValue:600})
    SectionLength!: number;

    @Field(() => Int, {nullable:false,defaultValue:10})
    MinSectionLength!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    FrameSkip!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MotionFrameSkip!: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    AnalysisFPSLimit!: any | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AnalysisUpdateDelay!: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS!: any | null;

    @Field(() => GraphQLDecimal, {nullable:true})
    AlarmMaxFPS!: any | null;

    @Field(() => Int, {nullable:false,defaultValue:250})
    FPSReportInterval!: number;

    @Field(() => Int, {nullable:false,defaultValue:6})
    RefBlendPerc!: number;

    @Field(() => Int, {nullable:false,defaultValue:6})
    AlarmRefBlendPerc!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Controllable!: number;

    @Field(() => Int, {nullable:true})
    ControlId!: number | null;

    @Field(() => String, {nullable:true})
    ControlDevice!: string | null;

    @Field(() => String, {nullable:true})
    ControlAddress!: string | null;

    @Field(() => GraphQLDecimal, {nullable:true})
    AutoStopTimeout!: any | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    TrackMotion!: number;

    @Field(() => Int, {nullable:true})
    TrackDelay!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:-1})
    ReturnLocation!: number;

    @Field(() => Int, {nullable:true})
    ReturnDelay!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    ModectDuringPTZ!: number;

    @Field(() => Int, {nullable:false,defaultValue:100})
    DefaultRate!: number;

    @Field(() => Int, {nullable:false,defaultValue:100})
    DefaultScale!: number;

    @Field(() => Monitors_DefaultCodec, {nullable:false,defaultValue:'auto'})
    DefaultCodec!: keyof typeof Monitors_DefaultCodec;

    @Field(() => Int, {nullable:false,defaultValue:0})
    SignalCheckPoints!: number;

    @Field(() => String, {nullable:false,defaultValue:'#0000BE'})
    SignalCheckColour!: string;

    @Field(() => String, {nullable:false,defaultValue:'red'})
    WebColour!: string;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Exif!: number;

    @Field(() => Int, {nullable:true})
    Sequence!: number | null;

    @Field(() => Int, {nullable:true})
    TotalEvents!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    ZoneCount!: number;

    @Field(() => String, {nullable:true})
    TotalEventDiskSpace!: bigint | null;

    @Field(() => Int, {nullable:true})
    Refresh!: number | null;

    @Field(() => GraphQLDecimal, {nullable:true})
    Latitude!: any | null;

    @Field(() => GraphQLDecimal, {nullable:true})
    Longitude!: any | null;

    @Field(() => Boolean, {nullable:false,defaultValue:false})
    RTSPServer!: boolean;

    @Field(() => String, {nullable:false,defaultValue:''})
    RTSPStreamName!: string;

    @Field(() => Monitors_Importance, {nullable:true})
    Importance!: keyof typeof Monitors_Importance | null;
}
