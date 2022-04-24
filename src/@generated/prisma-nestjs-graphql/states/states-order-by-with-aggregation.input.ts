import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { StatesCountOrderByAggregateInput } from './states-count-order-by-aggregate.input';
import { StatesAvgOrderByAggregateInput } from './states-avg-order-by-aggregate.input';
import { StatesMaxOrderByAggregateInput } from './states-max-order-by-aggregate.input';
import { StatesMinOrderByAggregateInput } from './states-min-order-by-aggregate.input';
import { StatesSumOrderByAggregateInput } from './states-sum-order-by-aggregate.input';

@InputType()
export class StatesOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Definition?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    IsActive?: keyof typeof SortOrder;

    @Field(() => StatesCountOrderByAggregateInput, {nullable:true})
    _count?: StatesCountOrderByAggregateInput;

    @Field(() => StatesAvgOrderByAggregateInput, {nullable:true})
    _avg?: StatesAvgOrderByAggregateInput;

    @Field(() => StatesMaxOrderByAggregateInput, {nullable:true})
    _max?: StatesMaxOrderByAggregateInput;

    @Field(() => StatesMinOrderByAggregateInput, {nullable:true})
    _min?: StatesMinOrderByAggregateInput;

    @Field(() => StatesSumOrderByAggregateInput, {nullable:true})
    _sum?: StatesSumOrderByAggregateInput;
}
