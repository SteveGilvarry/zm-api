import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsWhereInput } from './stats-where.input';
import { StatsOrderByWithRelationInput } from './stats-order-by-with-relation.input';
import { StatsWhereUniqueInput } from './stats-where-unique.input';
import { Int } from '@nestjs/graphql';
import { StatsCountAggregateInput } from './stats-count-aggregate.input';
import { StatsAvgAggregateInput } from './stats-avg-aggregate.input';
import { StatsSumAggregateInput } from './stats-sum-aggregate.input';
import { StatsMinAggregateInput } from './stats-min-aggregate.input';
import { StatsMaxAggregateInput } from './stats-max-aggregate.input';

@ArgsType()
export class StatsAggregateArgs {

    @Field(() => StatsWhereInput, {nullable:true})
    where?: StatsWhereInput;

    @Field(() => [StatsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<StatsOrderByWithRelationInput>;

    @Field(() => StatsWhereUniqueInput, {nullable:true})
    cursor?: StatsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => StatsCountAggregateInput, {nullable:true})
    _count?: StatsCountAggregateInput;

    @Field(() => StatsAvgAggregateInput, {nullable:true})
    _avg?: StatsAvgAggregateInput;

    @Field(() => StatsSumAggregateInput, {nullable:true})
    _sum?: StatsSumAggregateInput;

    @Field(() => StatsMinAggregateInput, {nullable:true})
    _min?: StatsMinAggregateInput;

    @Field(() => StatsMaxAggregateInput, {nullable:true})
    _max?: StatsMaxAggregateInput;
}
