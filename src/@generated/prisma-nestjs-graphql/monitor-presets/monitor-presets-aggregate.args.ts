import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereInput } from './monitor-presets-where.input';
import { MonitorPresetsOrderByWithRelationInput } from './monitor-presets-order-by-with-relation.input';
import { MonitorPresetsWhereUniqueInput } from './monitor-presets-where-unique.input';
import { Int } from '@nestjs/graphql';
import { MonitorPresetsCountAggregateInput } from './monitor-presets-count-aggregate.input';
import { MonitorPresetsAvgAggregateInput } from './monitor-presets-avg-aggregate.input';
import { MonitorPresetsSumAggregateInput } from './monitor-presets-sum-aggregate.input';
import { MonitorPresetsMinAggregateInput } from './monitor-presets-min-aggregate.input';
import { MonitorPresetsMaxAggregateInput } from './monitor-presets-max-aggregate.input';

@ArgsType()
export class MonitorPresetsAggregateArgs {

    @Field(() => MonitorPresetsWhereInput, {nullable:true})
    where?: MonitorPresetsWhereInput;

    @Field(() => [MonitorPresetsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<MonitorPresetsOrderByWithRelationInput>;

    @Field(() => MonitorPresetsWhereUniqueInput, {nullable:true})
    cursor?: MonitorPresetsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => MonitorPresetsCountAggregateInput, {nullable:true})
    _count?: MonitorPresetsCountAggregateInput;

    @Field(() => MonitorPresetsAvgAggregateInput, {nullable:true})
    _avg?: MonitorPresetsAvgAggregateInput;

    @Field(() => MonitorPresetsSumAggregateInput, {nullable:true})
    _sum?: MonitorPresetsSumAggregateInput;

    @Field(() => MonitorPresetsMinAggregateInput, {nullable:true})
    _min?: MonitorPresetsMinAggregateInput;

    @Field(() => MonitorPresetsMaxAggregateInput, {nullable:true})
    _max?: MonitorPresetsMaxAggregateInput;
}
