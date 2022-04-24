import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ControlsMaxAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Type?: true;

    @Field(() => Boolean, {nullable:true})
    Protocol?: true;

    @Field(() => Boolean, {nullable:true})
    CanWake?: true;

    @Field(() => Boolean, {nullable:true})
    CanSleep?: true;

    @Field(() => Boolean, {nullable:true})
    CanReset?: true;

    @Field(() => Boolean, {nullable:true})
    CanReboot?: true;

    @Field(() => Boolean, {nullable:true})
    CanZoom?: true;

    @Field(() => Boolean, {nullable:true})
    CanAutoZoom?: true;

    @Field(() => Boolean, {nullable:true})
    CanZoomAbs?: true;

    @Field(() => Boolean, {nullable:true})
    CanZoomRel?: true;

    @Field(() => Boolean, {nullable:true})
    CanZoomCon?: true;

    @Field(() => Boolean, {nullable:true})
    MinZoomRange?: true;

    @Field(() => Boolean, {nullable:true})
    MaxZoomRange?: true;

    @Field(() => Boolean, {nullable:true})
    MinZoomStep?: true;

    @Field(() => Boolean, {nullable:true})
    MaxZoomStep?: true;

    @Field(() => Boolean, {nullable:true})
    HasZoomSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MinZoomSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MaxZoomSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    CanFocus?: true;

    @Field(() => Boolean, {nullable:true})
    CanAutoFocus?: true;

    @Field(() => Boolean, {nullable:true})
    CanFocusAbs?: true;

    @Field(() => Boolean, {nullable:true})
    CanFocusRel?: true;

    @Field(() => Boolean, {nullable:true})
    CanFocusCon?: true;

    @Field(() => Boolean, {nullable:true})
    MinFocusRange?: true;

    @Field(() => Boolean, {nullable:true})
    MaxFocusRange?: true;

    @Field(() => Boolean, {nullable:true})
    MinFocusStep?: true;

    @Field(() => Boolean, {nullable:true})
    MaxFocusStep?: true;

    @Field(() => Boolean, {nullable:true})
    HasFocusSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MinFocusSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MaxFocusSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    CanIris?: true;

    @Field(() => Boolean, {nullable:true})
    CanAutoIris?: true;

    @Field(() => Boolean, {nullable:true})
    CanIrisAbs?: true;

    @Field(() => Boolean, {nullable:true})
    CanIrisRel?: true;

    @Field(() => Boolean, {nullable:true})
    CanIrisCon?: true;

    @Field(() => Boolean, {nullable:true})
    MinIrisRange?: true;

    @Field(() => Boolean, {nullable:true})
    MaxIrisRange?: true;

    @Field(() => Boolean, {nullable:true})
    MinIrisStep?: true;

    @Field(() => Boolean, {nullable:true})
    MaxIrisStep?: true;

    @Field(() => Boolean, {nullable:true})
    HasIrisSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MinIrisSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MaxIrisSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    CanGain?: true;

    @Field(() => Boolean, {nullable:true})
    CanAutoGain?: true;

    @Field(() => Boolean, {nullable:true})
    CanGainAbs?: true;

    @Field(() => Boolean, {nullable:true})
    CanGainRel?: true;

    @Field(() => Boolean, {nullable:true})
    CanGainCon?: true;

    @Field(() => Boolean, {nullable:true})
    MinGainRange?: true;

    @Field(() => Boolean, {nullable:true})
    MaxGainRange?: true;

    @Field(() => Boolean, {nullable:true})
    MinGainStep?: true;

    @Field(() => Boolean, {nullable:true})
    MaxGainStep?: true;

    @Field(() => Boolean, {nullable:true})
    HasGainSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MinGainSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MaxGainSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    CanWhite?: true;

    @Field(() => Boolean, {nullable:true})
    CanAutoWhite?: true;

    @Field(() => Boolean, {nullable:true})
    CanWhiteAbs?: true;

    @Field(() => Boolean, {nullable:true})
    CanWhiteRel?: true;

    @Field(() => Boolean, {nullable:true})
    CanWhiteCon?: true;

    @Field(() => Boolean, {nullable:true})
    MinWhiteRange?: true;

    @Field(() => Boolean, {nullable:true})
    MaxWhiteRange?: true;

    @Field(() => Boolean, {nullable:true})
    MinWhiteStep?: true;

    @Field(() => Boolean, {nullable:true})
    MaxWhiteStep?: true;

    @Field(() => Boolean, {nullable:true})
    HasWhiteSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MinWhiteSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MaxWhiteSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    HasPresets?: true;

    @Field(() => Boolean, {nullable:true})
    NumPresets?: true;

    @Field(() => Boolean, {nullable:true})
    HasHomePreset?: true;

    @Field(() => Boolean, {nullable:true})
    CanSetPresets?: true;

    @Field(() => Boolean, {nullable:true})
    CanMove?: true;

    @Field(() => Boolean, {nullable:true})
    CanMoveDiag?: true;

    @Field(() => Boolean, {nullable:true})
    CanMoveMap?: true;

    @Field(() => Boolean, {nullable:true})
    CanMoveAbs?: true;

    @Field(() => Boolean, {nullable:true})
    CanMoveRel?: true;

    @Field(() => Boolean, {nullable:true})
    CanMoveCon?: true;

    @Field(() => Boolean, {nullable:true})
    CanPan?: true;

    @Field(() => Boolean, {nullable:true})
    MinPanRange?: true;

    @Field(() => Boolean, {nullable:true})
    MaxPanRange?: true;

    @Field(() => Boolean, {nullable:true})
    MinPanStep?: true;

    @Field(() => Boolean, {nullable:true})
    MaxPanStep?: true;

    @Field(() => Boolean, {nullable:true})
    HasPanSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MinPanSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MaxPanSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    HasTurboPan?: true;

    @Field(() => Boolean, {nullable:true})
    TurboPanSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    CanTilt?: true;

    @Field(() => Boolean, {nullable:true})
    MinTiltRange?: true;

    @Field(() => Boolean, {nullable:true})
    MaxTiltRange?: true;

    @Field(() => Boolean, {nullable:true})
    MinTiltStep?: true;

    @Field(() => Boolean, {nullable:true})
    MaxTiltStep?: true;

    @Field(() => Boolean, {nullable:true})
    HasTiltSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MinTiltSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    MaxTiltSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    HasTurboTilt?: true;

    @Field(() => Boolean, {nullable:true})
    TurboTiltSpeed?: true;

    @Field(() => Boolean, {nullable:true})
    CanAutoScan?: true;

    @Field(() => Boolean, {nullable:true})
    NumScanPaths?: true;
}
