import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Controls_Type } from '../prisma/controls-type.enum';
import { ControlsCountAggregate } from './controls-count-aggregate.output';
import { ControlsAvgAggregate } from './controls-avg-aggregate.output';
import { ControlsSumAggregate } from './controls-sum-aggregate.output';
import { ControlsMinAggregate } from './controls-min-aggregate.output';
import { ControlsMaxAggregate } from './controls-max-aggregate.output';

@ObjectType()
export class ControlsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Controls_Type, {nullable:false})
    Type!: keyof typeof Controls_Type;

    @Field(() => String, {nullable:true})
    Protocol?: string;

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

    @Field(() => Int, {nullable:true})
    MinZoomRange?: number;

    @Field(() => Int, {nullable:true})
    MaxZoomRange?: number;

    @Field(() => Int, {nullable:true})
    MinZoomStep?: number;

    @Field(() => Int, {nullable:true})
    MaxZoomStep?: number;

    @Field(() => Int, {nullable:false})
    HasZoomSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinZoomSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxZoomSpeed?: number;

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

    @Field(() => Int, {nullable:true})
    MinFocusRange?: number;

    @Field(() => Int, {nullable:true})
    MaxFocusRange?: number;

    @Field(() => Int, {nullable:true})
    MinFocusStep?: number;

    @Field(() => Int, {nullable:true})
    MaxFocusStep?: number;

    @Field(() => Int, {nullable:false})
    HasFocusSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinFocusSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxFocusSpeed?: number;

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

    @Field(() => Int, {nullable:true})
    MinIrisRange?: number;

    @Field(() => Int, {nullable:true})
    MaxIrisRange?: number;

    @Field(() => Int, {nullable:true})
    MinIrisStep?: number;

    @Field(() => Int, {nullable:true})
    MaxIrisStep?: number;

    @Field(() => Int, {nullable:false})
    HasIrisSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinIrisSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxIrisSpeed?: number;

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

    @Field(() => Int, {nullable:true})
    MinGainRange?: number;

    @Field(() => Int, {nullable:true})
    MaxGainRange?: number;

    @Field(() => Int, {nullable:true})
    MinGainStep?: number;

    @Field(() => Int, {nullable:true})
    MaxGainStep?: number;

    @Field(() => Int, {nullable:false})
    HasGainSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinGainSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxGainSpeed?: number;

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

    @Field(() => Int, {nullable:true})
    MinWhiteRange?: number;

    @Field(() => Int, {nullable:true})
    MaxWhiteRange?: number;

    @Field(() => Int, {nullable:true})
    MinWhiteStep?: number;

    @Field(() => Int, {nullable:true})
    MaxWhiteStep?: number;

    @Field(() => Int, {nullable:false})
    HasWhiteSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinWhiteSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxWhiteSpeed?: number;

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

    @Field(() => Int, {nullable:true})
    MinPanRange?: number;

    @Field(() => Int, {nullable:true})
    MaxPanRange?: number;

    @Field(() => Int, {nullable:true})
    MinPanStep?: number;

    @Field(() => Int, {nullable:true})
    MaxPanStep?: number;

    @Field(() => Int, {nullable:false})
    HasPanSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinPanSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxPanSpeed?: number;

    @Field(() => Int, {nullable:false})
    HasTurboPan!: number;

    @Field(() => Int, {nullable:true})
    TurboPanSpeed?: number;

    @Field(() => Int, {nullable:false})
    CanTilt!: number;

    @Field(() => Int, {nullable:true})
    MinTiltRange?: number;

    @Field(() => Int, {nullable:true})
    MaxTiltRange?: number;

    @Field(() => Int, {nullable:true})
    MinTiltStep?: number;

    @Field(() => Int, {nullable:true})
    MaxTiltStep?: number;

    @Field(() => Int, {nullable:false})
    HasTiltSpeed!: number;

    @Field(() => Int, {nullable:true})
    MinTiltSpeed?: number;

    @Field(() => Int, {nullable:true})
    MaxTiltSpeed?: number;

    @Field(() => Int, {nullable:false})
    HasTurboTilt!: number;

    @Field(() => Int, {nullable:true})
    TurboTiltSpeed?: number;

    @Field(() => Int, {nullable:false})
    CanAutoScan!: number;

    @Field(() => Int, {nullable:false})
    NumScanPaths!: number;

    @Field(() => ControlsCountAggregate, {nullable:true})
    _count?: ControlsCountAggregate;

    @Field(() => ControlsAvgAggregate, {nullable:true})
    _avg?: ControlsAvgAggregate;

    @Field(() => ControlsSumAggregate, {nullable:true})
    _sum?: ControlsSumAggregate;

    @Field(() => ControlsMinAggregate, {nullable:true})
    _min?: ControlsMinAggregate;

    @Field(() => ControlsMaxAggregate, {nullable:true})
    _max?: ControlsMaxAggregate;
}
