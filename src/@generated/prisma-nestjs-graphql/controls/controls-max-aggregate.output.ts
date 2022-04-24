import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Controls_Type } from '../prisma/controls-type.enum';

@ObjectType()
export class ControlsMaxAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => Controls_Type, {nullable:true})
    Type?: keyof typeof Controls_Type;

    @Field(() => String, {nullable:true})
    Protocol?: string;

    @Field(() => Int, {nullable:true})
    CanWake?: number;

    @Field(() => Int, {nullable:true})
    CanSleep?: number;

    @Field(() => Int, {nullable:true})
    CanReset?: number;

    @Field(() => Int, {nullable:true})
    CanReboot?: number;

    @Field(() => Int, {nullable:true})
    CanZoom?: number;

    @Field(() => Int, {nullable:true})
    CanAutoZoom?: number;

    @Field(() => Int, {nullable:true})
    CanZoomAbs?: number;

    @Field(() => Int, {nullable:true})
    CanZoomRel?: number;

    @Field(() => Int, {nullable:true})
    CanZoomCon?: number;

    @Field(() => Int, {nullable:true})
    MinZoomRange?: number;

    @Field(() => Int, {nullable:true})
    MaxZoomRange?: number;

    @Field(() => Int, {nullable:true})
    MinZoomStep?: number;

    @Field(() => Int, {nullable:true})
    MaxZoomStep?: number;

    @Field(() => Int, {nullable:true})
    HasZoomSpeed?: number;

    @Field(() => Int, {nullable:true})
    MinZoomSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxZoomSpeed?: number;

    @Field(() => Int, {nullable:true})
    CanFocus?: number;

    @Field(() => Int, {nullable:true})
    CanAutoFocus?: number;

    @Field(() => Int, {nullable:true})
    CanFocusAbs?: number;

    @Field(() => Int, {nullable:true})
    CanFocusRel?: number;

    @Field(() => Int, {nullable:true})
    CanFocusCon?: number;

    @Field(() => Int, {nullable:true})
    MinFocusRange?: number;

    @Field(() => Int, {nullable:true})
    MaxFocusRange?: number;

    @Field(() => Int, {nullable:true})
    MinFocusStep?: number;

    @Field(() => Int, {nullable:true})
    MaxFocusStep?: number;

    @Field(() => Int, {nullable:true})
    HasFocusSpeed?: number;

    @Field(() => Int, {nullable:true})
    MinFocusSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxFocusSpeed?: number;

    @Field(() => Int, {nullable:true})
    CanIris?: number;

    @Field(() => Int, {nullable:true})
    CanAutoIris?: number;

    @Field(() => Int, {nullable:true})
    CanIrisAbs?: number;

    @Field(() => Int, {nullable:true})
    CanIrisRel?: number;

    @Field(() => Int, {nullable:true})
    CanIrisCon?: number;

    @Field(() => Int, {nullable:true})
    MinIrisRange?: number;

    @Field(() => Int, {nullable:true})
    MaxIrisRange?: number;

    @Field(() => Int, {nullable:true})
    MinIrisStep?: number;

    @Field(() => Int, {nullable:true})
    MaxIrisStep?: number;

    @Field(() => Int, {nullable:true})
    HasIrisSpeed?: number;

    @Field(() => Int, {nullable:true})
    MinIrisSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxIrisSpeed?: number;

    @Field(() => Int, {nullable:true})
    CanGain?: number;

    @Field(() => Int, {nullable:true})
    CanAutoGain?: number;

    @Field(() => Int, {nullable:true})
    CanGainAbs?: number;

    @Field(() => Int, {nullable:true})
    CanGainRel?: number;

    @Field(() => Int, {nullable:true})
    CanGainCon?: number;

    @Field(() => Int, {nullable:true})
    MinGainRange?: number;

    @Field(() => Int, {nullable:true})
    MaxGainRange?: number;

    @Field(() => Int, {nullable:true})
    MinGainStep?: number;

    @Field(() => Int, {nullable:true})
    MaxGainStep?: number;

    @Field(() => Int, {nullable:true})
    HasGainSpeed?: number;

    @Field(() => Int, {nullable:true})
    MinGainSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxGainSpeed?: number;

    @Field(() => Int, {nullable:true})
    CanWhite?: number;

    @Field(() => Int, {nullable:true})
    CanAutoWhite?: number;

    @Field(() => Int, {nullable:true})
    CanWhiteAbs?: number;

    @Field(() => Int, {nullable:true})
    CanWhiteRel?: number;

    @Field(() => Int, {nullable:true})
    CanWhiteCon?: number;

    @Field(() => Int, {nullable:true})
    MinWhiteRange?: number;

    @Field(() => Int, {nullable:true})
    MaxWhiteRange?: number;

    @Field(() => Int, {nullable:true})
    MinWhiteStep?: number;

    @Field(() => Int, {nullable:true})
    MaxWhiteStep?: number;

    @Field(() => Int, {nullable:true})
    HasWhiteSpeed?: number;

    @Field(() => Int, {nullable:true})
    MinWhiteSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxWhiteSpeed?: number;

    @Field(() => Int, {nullable:true})
    HasPresets?: number;

    @Field(() => Int, {nullable:true})
    NumPresets?: number;

    @Field(() => Int, {nullable:true})
    HasHomePreset?: number;

    @Field(() => Int, {nullable:true})
    CanSetPresets?: number;

    @Field(() => Int, {nullable:true})
    CanMove?: number;

    @Field(() => Int, {nullable:true})
    CanMoveDiag?: number;

    @Field(() => Int, {nullable:true})
    CanMoveMap?: number;

    @Field(() => Int, {nullable:true})
    CanMoveAbs?: number;

    @Field(() => Int, {nullable:true})
    CanMoveRel?: number;

    @Field(() => Int, {nullable:true})
    CanMoveCon?: number;

    @Field(() => Int, {nullable:true})
    CanPan?: number;

    @Field(() => Int, {nullable:true})
    MinPanRange?: number;

    @Field(() => Int, {nullable:true})
    MaxPanRange?: number;

    @Field(() => Int, {nullable:true})
    MinPanStep?: number;

    @Field(() => Int, {nullable:true})
    MaxPanStep?: number;

    @Field(() => Int, {nullable:true})
    HasPanSpeed?: number;

    @Field(() => Int, {nullable:true})
    MinPanSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxPanSpeed?: number;

    @Field(() => Int, {nullable:true})
    HasTurboPan?: number;

    @Field(() => Int, {nullable:true})
    TurboPanSpeed?: number;

    @Field(() => Int, {nullable:true})
    CanTilt?: number;

    @Field(() => Int, {nullable:true})
    MinTiltRange?: number;

    @Field(() => Int, {nullable:true})
    MaxTiltRange?: number;

    @Field(() => Int, {nullable:true})
    MinTiltStep?: number;

    @Field(() => Int, {nullable:true})
    MaxTiltStep?: number;

    @Field(() => Int, {nullable:true})
    HasTiltSpeed?: number;

    @Field(() => Int, {nullable:true})
    MinTiltSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxTiltSpeed?: number;

    @Field(() => Int, {nullable:true})
    HasTurboTilt?: number;

    @Field(() => Int, {nullable:true})
    TurboTiltSpeed?: number;

    @Field(() => Int, {nullable:true})
    CanAutoScan?: number;

    @Field(() => Int, {nullable:true})
    NumScanPaths?: number;
}
