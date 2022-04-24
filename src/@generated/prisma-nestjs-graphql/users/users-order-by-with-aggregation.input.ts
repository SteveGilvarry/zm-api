import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { UsersCountOrderByAggregateInput } from './users-count-order-by-aggregate.input';
import { UsersAvgOrderByAggregateInput } from './users-avg-order-by-aggregate.input';
import { UsersMaxOrderByAggregateInput } from './users-max-order-by-aggregate.input';
import { UsersMinOrderByAggregateInput } from './users-min-order-by-aggregate.input';
import { UsersSumOrderByAggregateInput } from './users-sum-order-by-aggregate.input';

@InputType()
export class UsersOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Username?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Password?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Language?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Enabled?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Stream?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Events?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Control?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Monitors?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Groups?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Devices?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Snapshots?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    System?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxBandwidth?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorIds?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TokenMinExpiry?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    APIEnabled?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HomeView?: keyof typeof SortOrder;

    @Field(() => UsersCountOrderByAggregateInput, {nullable:true})
    _count?: UsersCountOrderByAggregateInput;

    @Field(() => UsersAvgOrderByAggregateInput, {nullable:true})
    _avg?: UsersAvgOrderByAggregateInput;

    @Field(() => UsersMaxOrderByAggregateInput, {nullable:true})
    _max?: UsersMaxOrderByAggregateInput;

    @Field(() => UsersMinOrderByAggregateInput, {nullable:true})
    _min?: UsersMinOrderByAggregateInput;

    @Field(() => UsersSumOrderByAggregateInput, {nullable:true})
    _sum?: UsersSumOrderByAggregateInput;
}
