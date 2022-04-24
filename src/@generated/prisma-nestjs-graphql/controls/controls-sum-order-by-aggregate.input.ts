import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class ControlsSumOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanWake?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanSleep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanReset?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanReboot?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanZoom?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanAutoZoom?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanZoomAbs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanZoomRel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanZoomCon?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinZoomRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxZoomRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinZoomStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxZoomStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasZoomSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinZoomSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxZoomSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanFocus?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanAutoFocus?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanFocusAbs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanFocusRel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanFocusCon?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinFocusRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxFocusRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinFocusStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxFocusStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasFocusSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinFocusSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxFocusSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanIris?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanAutoIris?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanIrisAbs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanIrisRel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanIrisCon?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinIrisRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxIrisRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinIrisStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxIrisStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasIrisSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinIrisSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxIrisSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanGain?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanAutoGain?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanGainAbs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanGainRel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanGainCon?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinGainRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxGainRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinGainStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxGainStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasGainSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinGainSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxGainSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanWhite?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanAutoWhite?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanWhiteAbs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanWhiteRel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanWhiteCon?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinWhiteRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxWhiteRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinWhiteStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxWhiteStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasWhiteSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinWhiteSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxWhiteSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasPresets?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    NumPresets?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasHomePreset?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanSetPresets?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanMove?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanMoveDiag?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanMoveMap?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanMoveAbs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanMoveRel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanMoveCon?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanPan?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinPanRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxPanRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinPanStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxPanStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasPanSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinPanSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxPanSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasTurboPan?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TurboPanSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanTilt?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinTiltRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxTiltRange?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinTiltStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxTiltStep?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasTiltSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinTiltSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxTiltSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HasTurboTilt?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TurboTiltSpeed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CanAutoScan?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    NumScanPaths?: keyof typeof SortOrder;
}
