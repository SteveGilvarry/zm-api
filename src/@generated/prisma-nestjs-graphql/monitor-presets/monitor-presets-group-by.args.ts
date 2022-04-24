import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereInput } from './monitor-presets-where.input';
import { MonitorPresetsOrderByWithAggregationInput } from './monitor-presets-order-by-with-aggregation.input';
import { MonitorPresetsScalarFieldEnum } from './monitor-presets-scalar-field.enum';
import { MonitorPresetsScalarWhereWithAggregatesInput } from './monitor-presets-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { MonitorPresetsCountAggregateInput } from './monitor-presets-count-aggregate.input';
import { MonitorPresetsAvgAggregateInput } from './monitor-presets-avg-aggregate.input';
import { MonitorPresetsSumAggregateInput } from './monitor-presets-sum-aggregate.input';
import { MonitorPresetsMinAggregateInput } from './monitor-presets-min-aggregate.input';
import { MonitorPresetsMaxAggregateInput } from './monitor-presets-max-aggregate.input';

@ArgsType()
export class MonitorPresetsGroupByArgs {

    @Field(() => MonitorPresetsWhereInput, {nullable:true})
    where?: MonitorPresetsWhereInput;

    @Field(() => [MonitorPresetsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<MonitorPresetsOrderByWithAggregationInput>;

    @Field(() => [MonitorPresetsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof MonitorPresetsScalarFieldEnum>;

    @Field(() => MonitorPresetsScalarWhereWithAggregatesInput, {nullable:true})
    having?: MonitorPresetsScalarWhereWithAggregatesInput;

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
