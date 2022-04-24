import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereInput } from './monitors-where.input';
import { MonitorsOrderByWithAggregationInput } from './monitors-order-by-with-aggregation.input';
import { MonitorsScalarFieldEnum } from './monitors-scalar-field.enum';
import { MonitorsScalarWhereWithAggregatesInput } from './monitors-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { MonitorsCountAggregateInput } from './monitors-count-aggregate.input';
import { MonitorsAvgAggregateInput } from './monitors-avg-aggregate.input';
import { MonitorsSumAggregateInput } from './monitors-sum-aggregate.input';
import { MonitorsMinAggregateInput } from './monitors-min-aggregate.input';
import { MonitorsMaxAggregateInput } from './monitors-max-aggregate.input';

@ArgsType()
export class MonitorsGroupByArgs {

    @Field(() => MonitorsWhereInput, {nullable:true})
    where?: MonitorsWhereInput;

    @Field(() => [MonitorsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<MonitorsOrderByWithAggregationInput>;

    @Field(() => [MonitorsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof MonitorsScalarFieldEnum>;

    @Field(() => MonitorsScalarWhereWithAggregatesInput, {nullable:true})
    having?: MonitorsScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => MonitorsCountAggregateInput, {nullable:true})
    _count?: MonitorsCountAggregateInput;

    @Field(() => MonitorsAvgAggregateInput, {nullable:true})
    _avg?: MonitorsAvgAggregateInput;

    @Field(() => MonitorsSumAggregateInput, {nullable:true})
    _sum?: MonitorsSumAggregateInput;

    @Field(() => MonitorsMinAggregateInput, {nullable:true})
    _min?: MonitorsMinAggregateInput;

    @Field(() => MonitorsMaxAggregateInput, {nullable:true})
    _max?: MonitorsMaxAggregateInput;
}
