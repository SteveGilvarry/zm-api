import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ControlsCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Type!: number;

    @Field(() => Int, {nullable:false})
    Protocol!: number;

    @Field(() => Int, {nullable:false})
    CanWake!: number;

    @Field(() => Int, {nullable:false})
    CanSleep!: number;

    @Field(() => Int, {nullable:false})
    CanReset!: number;

    @Field(() => Int, {nullable:false})
    CanReboot!: number;

    @Field(() => Int, {nullable:false})
    CanZoom!: number;

    @Field(() => Int, {nullable:false})
    CanAutoZoom!: number;

    @Field(() => Int, {nullable:false})
    CanZoomAbs!: number;

    @Field(() => Int, {nullable:false})
    CanZoomRel!: number;

    @Field(() => Int, {nullable:false})
    CanZoomCon!: number;

    @Field(() => Int, {nullable:false})
    MinZoomRange!: number;

    @Field(() => Int, {nullable:false})
    MaxZoomRange!: number;

    @Field(() => Int, {nullable:false})
    MinZoomStep!: number;

    @Field(() => Int, {nullable:false})
    MaxZoomStep!: number;

    @Field(() => Int, {nullable:false})
    HasZoomSpeed!: number;

    @Field(() => Int, {nullable:false})
    MinZoomSpeed!: number;

    @Field(() => Int, {nullable:false})
    MaxZoomSpeed!: number;

    @Field(() => Int, {nullable:false})
    CanFocus!: number;

    @Field(() => Int, {nullable:false})
    CanAutoFocus!: number;

    @Field(() => Int, {nullable:false})
    CanFocusAbs!: number;

    @Field(() => Int, {nullable:false})
    CanFocusRel!: number;

    @Field(() => Int, {nullable:false})
    CanFocusCon!: number;

    @Field(() => Int, {nullable:false})
    MinFocusRange!: number;

    @Field(() => Int, {nullable:false})
    MaxFocusRange!: number;

    @Field(() => Int, {nullable:false})
    MinFocusStep!: number;

    @Field(() => Int, {nullable:false})
    MaxFocusStep!: number;

    @Field(() => Int, {nullable:false})
    HasFocusSpeed!: number;

    @Field(() => Int, {nullable:false})
    MinFocusSpeed!: number;

    @Field(() => Int, {nullable:false})
    MaxFocusSpeed!: number;

    @Field(() => Int, {nullable:false})
    CanIris!: number;

    @Field(() => Int, {nullable:false})
    CanAutoIris!: number;

    @Field(() => Int, {nullable:false})
    CanIrisAbs!: number;

    @Field(() => Int, {nullable:false})
    CanIrisRel!: number;

    @Field(() => Int, {nullable:false})
    CanIrisCon!: number;

    @Field(() => Int, {nullable:false})
    MinIrisRange!: number;

    @Field(() => Int, {nullable:false})
    MaxIrisRange!: number;

    @Field(() => Int, {nullable:false})
    MinIrisStep!: number;

    @Field(() => Int, {nullable:false})
    MaxIrisStep!: number;

    @Field(() => Int, {nullable:false})
    HasIrisSpeed!: number;

    @Field(() => Int, {nullable:false})
    MinIrisSpeed!: number;

    @Field(() => Int, {nullable:false})
    MaxIrisSpeed!: number;

    @Field(() => Int, {nullable:false})
    CanGain!: number;

    @Field(() => Int, {nullable:false})
    CanAutoGain!: number;

    @Field(() => Int, {nullable:false})
    CanGainAbs!: number;

    @Field(() => Int, {nullable:false})
    CanGainRel!: number;

    @Field(() => Int, {nullable:false})
    CanGainCon!: number;

    @Field(() => Int, {nullable:false})
    MinGainRange!: number;

    @Field(() => Int, {nullable:false})
    MaxGainRange!: number;

    @Field(() => Int, {nullable:false})
    MinGainStep!: number;

    @Field(() => Int, {nullable:false})
    MaxGainStep!: number;

    @Field(() => Int, {nullable:false})
    HasGainSpeed!: number;

    @Field(() => Int, {nullable:false})
    MinGainSpeed!: number;

    @Field(() => Int, {nullable:false})
    MaxGainSpeed!: number;

    @Field(() => Int, {nullable:false})
    CanWhite!: number;

    @Field(() => Int, {nullable:false})
    CanAutoWhite!: number;

    @Field(() => Int, {nullable:false})
    CanWhiteAbs!: number;

    @Field(() => Int, {nullable:false})
    CanWhiteRel!: number;

    @Field(() => Int, {nullable:false})
    CanWhiteCon!: number;

    @Field(() => Int, {nullable:false})
    MinWhiteRange!: number;

    @Field(() => Int, {nullable:false})
    MaxWhiteRange!: number;

    @Field(() => Int, {nullable:false})
    MinWhiteStep!: number;

    @Field(() => Int, {nullable:false})
    MaxWhiteStep!: number;

    @Field(() => Int, {nullable:false})
    HasWhiteSpeed!: number;

    @Field(() => Int, {nullable:false})
    MinWhiteSpeed!: number;

    @Field(() => Int, {nullable:false})
    MaxWhiteSpeed!: number;

    @Field(() => Int, {nullable:false})
    HasPresets!: number;

    @Field(() => Int, {nullable:false})
    NumPresets!: number;

    @Field(() => Int, {nullable:false})
    HasHomePreset!: number;

    @Field(() => Int, {nullable:false})
    CanSetPresets!: number;

    @Field(() => Int, {nullable:false})
    CanMove!: number;

    @Field(() => Int, {nullable:false})
    CanMoveDiag!: number;

    @Field(() => Int, {nullable:false})
    CanMoveMap!: number;

    @Field(() => Int, {nullable:false})
    CanMoveAbs!: number;

    @Field(() => Int, {nullable:false})
    CanMoveRel!: number;

    @Field(() => Int, {nullable:false})
    CanMoveCon!: number;

    @Field(() => Int, {nullable:false})
    CanPan!: number;

    @Field(() => Int, {nullable:false})
    MinPanRange!: number;

    @Field(() => Int, {nullable:false})
    MaxPanRange!: number;

    @Field(() => Int, {nullable:false})
    MinPanStep!: number;

    @Field(() => Int, {nullable:false})
    MaxPanStep!: number;

    @Field(() => Int, {nullable:false})
    HasPanSpeed!: number;

    @Field(() => Int, {nullable:false})
    MinPanSpeed!: number;

    @Field(() => Int, {nullable:false})
    MaxPanSpeed!: number;

    @Field(() => Int, {nullable:false})
    HasTurboPan!: number;

    @Field(() => Int, {nullable:false})
    TurboPanSpeed!: number;

    @Field(() => Int, {nullable:false})
    CanTilt!: number;

    @Field(() => Int, {nullable:false})
    MinTiltRange!: number;

    @Field(() => Int, {nullable:false})
    MaxTiltRange!: number;

    @Field(() => Int, {nullable:false})
    MinTiltStep!: number;

    @Field(() => Int, {nullable:false})
    MaxTiltStep!: number;

    @Field(() => Int, {nullable:false})
    HasTiltSpeed!: number;

    @Field(() => Int, {nullable:false})
    MinTiltSpeed!: number;

    @Field(() => Int, {nullable:false})
    MaxTiltSpeed!: number;

    @Field(() => Int, {nullable:false})
    HasTurboTilt!: number;

    @Field(() => Int, {nullable:false})
    TurboTiltSpeed!: number;

    @Field(() => Int, {nullable:false})
    CanAutoScan!: number;

    @Field(() => Int, {nullable:false})
    NumScanPaths!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
