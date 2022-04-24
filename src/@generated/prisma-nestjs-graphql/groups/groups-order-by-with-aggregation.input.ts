import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { GroupsCountOrderByAggregateInput } from './groups-count-order-by-aggregate.input';
import { GroupsAvgOrderByAggregateInput } from './groups-avg-order-by-aggregate.input';
import { GroupsMaxOrderByAggregateInput } from './groups-max-order-by-aggregate.input';
import { GroupsMinOrderByAggregateInput } from './groups-min-order-by-aggregate.input';
import { GroupsSumOrderByAggregateInput } from './groups-sum-order-by-aggregate.input';

@InputType()
export class GroupsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ParentId?: keyof typeof SortOrder;

    @Field(() => GroupsCountOrderByAggregateInput, {nullable:true})
    _count?: GroupsCountOrderByAggregateInput;

    @Field(() => GroupsAvgOrderByAggregateInput, {nullable:true})
    _avg?: GroupsAvgOrderByAggregateInput;

    @Field(() => GroupsMaxOrderByAggregateInput, {nullable:true})
    _max?: GroupsMaxOrderByAggregateInput;

    @Field(() => GroupsMinOrderByAggregateInput, {nullable:true})
    _min?: GroupsMinOrderByAggregateInput;

    @Field(() => GroupsSumOrderByAggregateInput, {nullable:true})
    _sum?: GroupsSumOrderByAggregateInput;
}
