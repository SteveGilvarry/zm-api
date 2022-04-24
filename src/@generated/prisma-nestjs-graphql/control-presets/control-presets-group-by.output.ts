import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { ControlPresetsCountAggregate } from './control-presets-count-aggregate.output';
import { ControlPresetsAvgAggregate } from './control-presets-avg-aggregate.output';
import { ControlPresetsSumAggregate } from './control-presets-sum-aggregate.output';
import { ControlPresetsMinAggregate } from './control-presets-min-aggregate.output';
import { ControlPresetsMaxAggregate } from './control-presets-max-aggregate.output';

@ObjectType()
export class ControlPresetsGroupBy {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    Preset!: number;

    @Field(() => String, {nullable:false})
    Label!: string;

    @Field(() => ControlPresetsCountAggregate, {nullable:true})
    _count?: ControlPresetsCountAggregate;

    @Field(() => ControlPresetsAvgAggregate, {nullable:true})
    _avg?: ControlPresetsAvgAggregate;

    @Field(() => ControlPresetsSumAggregate, {nullable:true})
    _sum?: ControlPresetsSumAggregate;

    @Field(() => ControlPresetsMinAggregate, {nullable:true})
    _min?: ControlPresetsMinAggregate;

    @Field(() => ControlPresetsMaxAggregate, {nullable:true})
    _max?: ControlPresetsMaxAggregate;
}
