import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereInput } from './monitors-where.input';
import { MonitorsOrderByWithRelationInput } from './monitors-order-by-with-relation.input';
import { MonitorsWhereUniqueInput } from './monitors-where-unique.input';
import { Int } from '@nestjs/graphql';
import { MonitorsCountAggregateInput } from './monitors-count-aggregate.input';
import { MonitorsAvgAggregateInput } from './monitors-avg-aggregate.input';
import { MonitorsSumAggregateInput } from './monitors-sum-aggregate.input';
import { MonitorsMinAggregateInput } from './monitors-min-aggregate.input';
import { MonitorsMaxAggregateInput } from './monitors-max-aggregate.input';

@ArgsType()
export class MonitorsAggregateArgs {

    @Field(() => MonitorsWhereInput, {nullable:true})
    where?: MonitorsWhereInput;

    @Field(() => [MonitorsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<MonitorsOrderByWithRelationInput>;

    @Field(() => MonitorsWhereUniqueInput, {nullable:true})
    cursor?: MonitorsWhereUniqueInput;

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
