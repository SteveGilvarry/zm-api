import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereInput } from './states-where.input';
import { Type } from 'class-transformer';
import { StatesOrderByWithRelationInput } from './states-order-by-with-relation.input';
import { StatesWhereUniqueInput } from './states-where-unique.input';
import { Int } from '@nestjs/graphql';
import { StatesCountAggregateInput } from './states-count-aggregate.input';
import { StatesAvgAggregateInput } from './states-avg-aggregate.input';
import { StatesSumAggregateInput } from './states-sum-aggregate.input';
import { StatesMinAggregateInput } from './states-min-aggregate.input';
import { StatesMaxAggregateInput } from './states-max-aggregate.input';

@ArgsType()
export class StatesAggregateArgs {

    @Field(() => StatesWhereInput, {nullable:true})
    @Type(() => StatesWhereInput)
    where?: StatesWhereInput;

    @Field(() => [StatesOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<StatesOrderByWithRelationInput>;

    @Field(() => StatesWhereUniqueInput, {nullable:true})
    cursor?: StatesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => StatesCountAggregateInput, {nullable:true})
    _count?: StatesCountAggregateInput;

    @Field(() => StatesAvgAggregateInput, {nullable:true})
    _avg?: StatesAvgAggregateInput;

    @Field(() => StatesSumAggregateInput, {nullable:true})
    _sum?: StatesSumAggregateInput;

    @Field(() => StatesMinAggregateInput, {nullable:true})
    _min?: StatesMinAggregateInput;

    @Field(() => StatesMaxAggregateInput, {nullable:true})
    _max?: StatesMaxAggregateInput;
}
