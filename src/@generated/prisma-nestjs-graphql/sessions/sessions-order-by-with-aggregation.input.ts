import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { SessionsCountOrderByAggregateInput } from './sessions-count-order-by-aggregate.input';
import { SessionsAvgOrderByAggregateInput } from './sessions-avg-order-by-aggregate.input';
import { SessionsMaxOrderByAggregateInput } from './sessions-max-order-by-aggregate.input';
import { SessionsMinOrderByAggregateInput } from './sessions-min-order-by-aggregate.input';
import { SessionsSumOrderByAggregateInput } from './sessions-sum-order-by-aggregate.input';

@InputType()
export class SessionsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    access?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    data?: keyof typeof SortOrder;

    @Field(() => SessionsCountOrderByAggregateInput, {nullable:true})
    _count?: SessionsCountOrderByAggregateInput;

    @Field(() => SessionsAvgOrderByAggregateInput, {nullable:true})
    _avg?: SessionsAvgOrderByAggregateInput;

    @Field(() => SessionsMaxOrderByAggregateInput, {nullable:true})
    _max?: SessionsMaxOrderByAggregateInput;

    @Field(() => SessionsMinOrderByAggregateInput, {nullable:true})
    _min?: SessionsMinOrderByAggregateInput;

    @Field(() => SessionsSumOrderByAggregateInput, {nullable:true})
    _sum?: SessionsSumOrderByAggregateInput;
}
