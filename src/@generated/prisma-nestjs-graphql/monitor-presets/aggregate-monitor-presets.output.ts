import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { MonitorPresetsCountAggregate } from './monitor-presets-count-aggregate.output';
import { MonitorPresetsAvgAggregate } from './monitor-presets-avg-aggregate.output';
import { MonitorPresetsSumAggregate } from './monitor-presets-sum-aggregate.output';
import { MonitorPresetsMinAggregate } from './monitor-presets-min-aggregate.output';
import { MonitorPresetsMaxAggregate } from './monitor-presets-max-aggregate.output';

@ObjectType()
export class AggregateMonitorPresets {

    @Field(() => MonitorPresetsCountAggregate, {nullable:true})
    _count?: MonitorPresetsCountAggregate;

    @Field(() => MonitorPresetsAvgAggregate, {nullable:true})
    _avg?: MonitorPresetsAvgAggregate;

    @Field(() => MonitorPresetsSumAggregate, {nullable:true})
    _sum?: MonitorPresetsSumAggregate;

    @Field(() => MonitorPresetsMinAggregate, {nullable:true})
    _min?: MonitorPresetsMinAggregate;

    @Field(() => MonitorPresetsMaxAggregate, {nullable:true})
    _max?: MonitorPresetsMaxAggregate;
}
