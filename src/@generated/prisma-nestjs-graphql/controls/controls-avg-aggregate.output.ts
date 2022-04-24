import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class ControlsAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    CanWake?: number;

    @Field(() => Float, {nullable:true})
    CanSleep?: number;

    @Field(() => Float, {nullable:true})
    CanReset?: number;

    @Field(() => Float, {nullable:true})
    CanReboot?: number;

    @Field(() => Float, {nullable:true})
    CanZoom?: number;

    @Field(() => Float, {nullable:true})
    CanAutoZoom?: number;

    @Field(() => Float, {nullable:true})
    CanZoomAbs?: number;

    @Field(() => Float, {nullable:true})
    CanZoomRel?: number;

    @Field(() => Float, {nullable:true})
    CanZoomCon?: number;

    @Field(() => Float, {nullable:true})
    MinZoomRange?: number;

    @Field(() => Float, {nullable:true})
    MaxZoomRange?: number;

    @Field(() => Float, {nullable:true})
    MinZoomStep?: number;

    @Field(() => Float, {nullable:true})
    MaxZoomStep?: number;

    @Field(() => Float, {nullable:true})
    HasZoomSpeed?: number;

    @Field(() => Float, {nullable:true})
    MinZoomSpeed?: number;

    @Field(() => Float, {nullable:true})
    MaxZoomSpeed?: number;

    @Field(() => Float, {nullable:true})
    CanFocus?: number;

    @Field(() => Float, {nullable:true})
    CanAutoFocus?: number;

    @Field(() => Float, {nullable:true})
    CanFocusAbs?: number;

    @Field(() => Float, {nullable:true})
    CanFocusRel?: number;

    @Field(() => Float, {nullable:true})
    CanFocusCon?: number;

    @Field(() => Float, {nullable:true})
    MinFocusRange?: number;

    @Field(() => Float, {nullable:true})
    MaxFocusRange?: number;

    @Field(() => Float, {nullable:true})
    MinFocusStep?: number;

    @Field(() => Float, {nullable:true})
    MaxFocusStep?: number;

    @Field(() => Float, {nullable:true})
    HasFocusSpeed?: number;

    @Field(() => Float, {nullable:true})
    MinFocusSpeed?: number;

    @Field(() => Float, {nullable:true})
    MaxFocusSpeed?: number;

    @Field(() => Float, {nullable:true})
    CanIris?: number;

    @Field(() => Float, {nullable:true})
    CanAutoIris?: number;

    @Field(() => Float, {nullable:true})
    CanIrisAbs?: number;

    @Field(() => Float, {nullable:true})
    CanIrisRel?: number;

    @Field(() => Float, {nullable:true})
    CanIrisCon?: number;

    @Field(() => Float, {nullable:true})
    MinIrisRange?: number;

    @Field(() => Float, {nullable:true})
    MaxIrisRange?: number;

    @Field(() => Float, {nullable:true})
    MinIrisStep?: number;

    @Field(() => Float, {nullable:true})
    MaxIrisStep?: number;

    @Field(() => Float, {nullable:true})
    HasIrisSpeed?: number;

    @Field(() => Float, {nullable:true})
    MinIrisSpeed?: number;

    @Field(() => Float, {nullable:true})
    MaxIrisSpeed?: number;

    @Field(() => Float, {nullable:true})
    CanGain?: number;

    @Field(() => Float, {nullable:true})
    CanAutoGain?: number;

    @Field(() => Float, {nullable:true})
    CanGainAbs?: number;

    @Field(() => Float, {nullable:true})
    CanGainRel?: number;

    @Field(() => Float, {nullable:true})
    CanGainCon?: number;

    @Field(() => Float, {nullable:true})
    MinGainRange?: number;

    @Field(() => Float, {nullable:true})
    MaxGainRange?: number;

    @Field(() => Float, {nullable:true})
    MinGainStep?: number;

    @Field(() => Float, {nullable:true})
    MaxGainStep?: number;

    @Field(() => Float, {nullable:true})
    HasGainSpeed?: number;

    @Field(() => Float, {nullable:true})
    MinGainSpeed?: number;

    @Field(() => Float, {nullable:true})
    MaxGainSpeed?: number;

    @Field(() => Float, {nullable:true})
    CanWhite?: number;

    @Field(() => Float, {nullable:true})
    CanAutoWhite?: number;

    @Field(() => Float, {nullable:true})
    CanWhiteAbs?: number;

    @Field(() => Float, {nullable:true})
    CanWhiteRel?: number;

    @Field(() => Float, {nullable:true})
    CanWhiteCon?: number;

    @Field(() => Float, {nullable:true})
    MinWhiteRange?: number;

    @Field(() => Float, {nullable:true})
    MaxWhiteRange?: number;

    @Field(() => Float, {nullable:true})
    MinWhiteStep?: number;

    @Field(() => Float, {nullable:true})
    MaxWhiteStep?: number;

    @Field(() => Float, {nullable:true})
    HasWhiteSpeed?: number;

    @Field(() => Float, {nullable:true})
    MinWhiteSpeed?: number;

    @Field(() => Float, {nullable:true})
    MaxWhiteSpeed?: number;

    @Field(() => Float, {nullable:true})
    HasPresets?: number;

    @Field(() => Float, {nullable:true})
    NumPresets?: number;

    @Field(() => Float, {nullable:true})
    HasHomePreset?: number;

    @Field(() => Float, {nullable:true})
    CanSetPresets?: number;

    @Field(() => Float, {nullable:true})
    CanMove?: number;

    @Field(() => Float, {nullable:true})
    CanMoveDiag?: number;

    @Field(() => Float, {nullable:true})
    CanMoveMap?: number;

    @Field(() => Float, {nullable:true})
    CanMoveAbs?: number;

    @Field(() => Float, {nullable:true})
    CanMoveRel?: number;

    @Field(() => Float, {nullable:true})
    CanMoveCon?: number;

    @Field(() => Float, {nullable:true})
    CanPan?: number;

    @Field(() => Float, {nullable:true})
    MinPanRange?: number;

    @Field(() => Float, {nullable:true})
    MaxPanRange?: number;

    @Field(() => Float, {nullable:true})
    MinPanStep?: number;

    @Field(() => Float, {nullable:true})
    MaxPanStep?: number;

    @Field(() => Float, {nullable:true})
    HasPanSpeed?: number;

    @Field(() => Float, {nullable:true})
    MinPanSpeed?: number;

    @Field(() => Float, {nullable:true})
    MaxPanSpeed?: number;

    @Field(() => Float, {nullable:true})
    HasTurboPan?: number;

    @Field(() => Float, {nullable:true})
    TurboPanSpeed?: number;

    @Field(() => Float, {nullable:true})
    CanTilt?: number;

    @Field(() => Float, {nullable:true})
    MinTiltRange?: number;

    @Field(() => Float, {nullable:true})
    MaxTiltRange?: number;

    @Field(() => Float, {nullable:true})
    MinTiltStep?: number;

    @Field(() => Float, {nullable:true})
    MaxTiltStep?: number;

    @Field(() => Float, {nullable:true})
    HasTiltSpeed?: number;

    @Field(() => Float, {nullable:true})
    MinTiltSpeed?: number;

    @Field(() => Float, {nullable:true})
    MaxTiltSpeed?: number;

    @Field(() => Float, {nullable:true})
    HasTurboTilt?: number;

    @Field(() => Float, {nullable:true})
    TurboTiltSpeed?: number;

    @Field(() => Float, {nullable:true})
    CanAutoScan?: number;

    @Field(() => Float, {nullable:true})
    NumScanPaths?: number;
}
