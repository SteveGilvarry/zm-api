import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { EnumControls_TypeFilter } from '../prisma/enum-controls-type-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';

@InputType()
export class ControlsWhereInput {

    @Field(() => [ControlsWhereInput], {nullable:true})
    AND?: Array<ControlsWhereInput>;

    @Field(() => [ControlsWhereInput], {nullable:true})
    OR?: Array<ControlsWhereInput>;

    @Field(() => [ControlsWhereInput], {nullable:true})
    NOT?: Array<ControlsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => EnumControls_TypeFilter, {nullable:true})
    Type?: EnumControls_TypeFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Protocol?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    CanWake?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanSleep?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanReset?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanReboot?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanZoom?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanAutoZoom?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanZoomAbs?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanZoomRel?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanZoomCon?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinZoomRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxZoomRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinZoomStep?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxZoomStep?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasZoomSpeed?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinZoomSpeed?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxZoomSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    CanFocus?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanAutoFocus?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanFocusAbs?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanFocusRel?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanFocusCon?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinFocusRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxFocusRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinFocusStep?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxFocusStep?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasFocusSpeed?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinFocusSpeed?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxFocusSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    CanIris?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanAutoIris?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanIrisAbs?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanIrisRel?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanIrisCon?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinIrisRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxIrisRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinIrisStep?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxIrisStep?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasIrisSpeed?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinIrisSpeed?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxIrisSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    CanGain?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanAutoGain?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanGainAbs?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanGainRel?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanGainCon?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinGainRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxGainRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinGainStep?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxGainStep?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasGainSpeed?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinGainSpeed?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxGainSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    CanWhite?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanAutoWhite?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanWhiteAbs?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanWhiteRel?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanWhiteCon?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinWhiteRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxWhiteRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinWhiteStep?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxWhiteStep?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasWhiteSpeed?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinWhiteSpeed?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxWhiteSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasPresets?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    NumPresets?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    HasHomePreset?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanSetPresets?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanMove?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanMoveDiag?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanMoveMap?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanMoveAbs?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanMoveRel?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanMoveCon?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    CanPan?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinPanRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxPanRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinPanStep?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxPanStep?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasPanSpeed?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinPanSpeed?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxPanSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasTurboPan?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    TurboPanSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    CanTilt?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinTiltRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxTiltRange?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinTiltStep?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxTiltStep?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasTiltSpeed?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinTiltSpeed?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxTiltSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    HasTurboTilt?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    TurboTiltSpeed?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    CanAutoScan?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    NumScanPaths?: IntFilter;
}
