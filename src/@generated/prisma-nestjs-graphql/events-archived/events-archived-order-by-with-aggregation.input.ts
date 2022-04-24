import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Events_ArchivedCountOrderByAggregateInput } from './events-archived-count-order-by-aggregate.input';
import { Events_ArchivedAvgOrderByAggregateInput } from './events-archived-avg-order-by-aggregate.input';
import { Events_ArchivedMaxOrderByAggregateInput } from './events-archived-max-order-by-aggregate.input';
import { Events_ArchivedMinOrderByAggregateInput } from './events-archived-min-order-by-aggregate.input';
import { Events_ArchivedSumOrderByAggregateInput } from './events-archived-sum-order-by-aggregate.input';

@InputType()
export class Events_ArchivedOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;

    @Field(() => Events_ArchivedCountOrderByAggregateInput, {nullable:true})
    _count?: Events_ArchivedCountOrderByAggregateInput;

    @Field(() => Events_ArchivedAvgOrderByAggregateInput, {nullable:true})
    _avg?: Events_ArchivedAvgOrderByAggregateInput;

    @Field(() => Events_ArchivedMaxOrderByAggregateInput, {nullable:true})
    _max?: Events_ArchivedMaxOrderByAggregateInput;

    @Field(() => Events_ArchivedMinOrderByAggregateInput, {nullable:true})
    _min?: Events_ArchivedMinOrderByAggregateInput;

    @Field(() => Events_ArchivedSumOrderByAggregateInput, {nullable:true})
    _sum?: Events_ArchivedSumOrderByAggregateInput;
}
