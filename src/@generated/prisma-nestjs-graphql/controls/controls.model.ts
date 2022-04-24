import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Controls_Type } from '../prisma/controls-type.enum';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Controls {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Controls_Type, {nullable:false,defaultValue:'Local'})
    Type!: keyof typeof Controls_Type;

    @Field(() => String, {nullable:true})
    Protocol!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanWake!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanSleep!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanReset!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanReboot!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanZoom!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanAutoZoom!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanZoomAbs!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanZoomRel!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanZoomCon!: number;

    @Field(() => Int, {nullable:true})
    MinZoomRange!: number | null;

    @Field(() => Int, {nullable:true})
    MaxZoomRange!: number | null;

    @Field(() => Int, {nullable:true})
    MinZoomStep!: number | null;

    @Field(() => Int, {nullable:true})
    MaxZoomStep!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasZoomSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinZoomSpeed!: number | null;

    @Field(() => Int, {nullable:true})
    MaxZoomSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanFocus!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanAutoFocus!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanFocusAbs!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanFocusRel!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanFocusCon!: number;

    @Field(() => Int, {nullable:true})
    MinFocusRange!: number | null;

    @Field(() => Int, {nullable:true})
    MaxFocusRange!: number | null;

    @Field(() => Int, {nullable:true})
    MinFocusStep!: number | null;

    @Field(() => Int, {nullable:true})
    MaxFocusStep!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasFocusSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinFocusSpeed!: number | null;

    @Field(() => Int, {nullable:true})
    MaxFocusSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanIris!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanAutoIris!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanIrisAbs!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanIrisRel!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanIrisCon!: number;

    @Field(() => Int, {nullable:true})
    MinIrisRange!: number | null;

    @Field(() => Int, {nullable:true})
    MaxIrisRange!: number | null;

    @Field(() => Int, {nullable:true})
    MinIrisStep!: number | null;

    @Field(() => Int, {nullable:true})
    MaxIrisStep!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasIrisSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinIrisSpeed!: number | null;

    @Field(() => Int, {nullable:true})
    MaxIrisSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanGain!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanAutoGain!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanGainAbs!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanGainRel!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanGainCon!: number;

    @Field(() => Int, {nullable:true})
    MinGainRange!: number | null;

    @Field(() => Int, {nullable:true})
    MaxGainRange!: number | null;

    @Field(() => Int, {nullable:true})
    MinGainStep!: number | null;

    @Field(() => Int, {nullable:true})
    MaxGainStep!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasGainSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinGainSpeed!: number | null;

    @Field(() => Int, {nullable:true})
    MaxGainSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanWhite!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanAutoWhite!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanWhiteAbs!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanWhiteRel!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanWhiteCon!: number;

    @Field(() => Int, {nullable:true})
    MinWhiteRange!: number | null;

    @Field(() => Int, {nullable:true})
    MaxWhiteRange!: number | null;

    @Field(() => Int, {nullable:true})
    MinWhiteStep!: number | null;

    @Field(() => Int, {nullable:true})
    MaxWhiteStep!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasWhiteSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinWhiteSpeed!: number | null;

    @Field(() => Int, {nullable:true})
    MaxWhiteSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasPresets!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    NumPresets!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasHomePreset!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanSetPresets!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanMove!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanMoveDiag!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanMoveMap!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanMoveAbs!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanMoveRel!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanMoveCon!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanPan!: number;

    @Field(() => Int, {nullable:true})
    MinPanRange!: number | null;

    @Field(() => Int, {nullable:true})
    MaxPanRange!: number | null;

    @Field(() => Int, {nullable:true})
    MinPanStep!: number | null;

    @Field(() => Int, {nullable:true})
    MaxPanStep!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasPanSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinPanSpeed!: number | null;

    @Field(() => Int, {nullable:true})
    MaxPanSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasTurboPan!: number;

    @Field(() => Int, {nullable:true})
    TurboPanSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanTilt!: number;

    @Field(() => Int, {nullable:true})
    MinTiltRange!: number | null;

    @Field(() => Int, {nullable:true})
    MaxTiltRange!: number | null;

    @Field(() => Int, {nullable:true})
    MinTiltStep!: number | null;

    @Field(() => Int, {nullable:true})
    MaxTiltStep!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasTiltSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinTiltSpeed!: number | null;

    @Field(() => Int, {nullable:true})
    MaxTiltSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    HasTurboTilt!: number;

    @Field(() => Int, {nullable:true})
    TurboTiltSpeed!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CanAutoScan!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    NumScanPaths!: number;
}
