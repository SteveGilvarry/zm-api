import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereInput } from './states-where.input';
import { StatesOrderByWithAggregationInput } from './states-order-by-with-aggregation.input';
import { StatesScalarFieldEnum } from './states-scalar-field.enum';
import { StatesScalarWhereWithAggregatesInput } from './states-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { StatesCountAggregateInput } from './states-count-aggregate.input';
import { StatesAvgAggregateInput } from './states-avg-aggregate.input';
import { StatesSumAggregateInput } from './states-sum-aggregate.input';
import { StatesMinAggregateInput } from './states-min-aggregate.input';
import { StatesMaxAggregateInput } from './states-max-aggregate.input';

@ArgsType()
export class StatesGroupByArgs {

    @Field(() => StatesWhereInput, {nullable:true})
    where?: StatesWhereInput;

    @Field(() => [StatesOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<StatesOrderByWithAggregationInput>;

    @Field(() => [StatesScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof StatesScalarFieldEnum>;

    @Field(() => StatesScalarWhereWithAggregatesInput, {nullable:true})
    having?: StatesScalarWhereWithAggregatesInput;

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
