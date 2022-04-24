import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Events_MonthCountOrderByAggregateInput } from './events-month-count-order-by-aggregate.input';
import { Events_MonthAvgOrderByAggregateInput } from './events-month-avg-order-by-aggregate.input';
import { Events_MonthMaxOrderByAggregateInput } from './events-month-max-order-by-aggregate.input';
import { Events_MonthMinOrderByAggregateInput } from './events-month-min-order-by-aggregate.input';
import { Events_MonthSumOrderByAggregateInput } from './events-month-sum-order-by-aggregate.input';

@InputType()
export class Events_MonthOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StartDateTime?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;

    @Field(() => Events_MonthCountOrderByAggregateInput, {nullable:true})
    _count?: Events_MonthCountOrderByAggregateInput;

    @Field(() => Events_MonthAvgOrderByAggregateInput, {nullable:true})
    _avg?: Events_MonthAvgOrderByAggregateInput;

    @Field(() => Events_MonthMaxOrderByAggregateInput, {nullable:true})
    _max?: Events_MonthMaxOrderByAggregateInput;

    @Field(() => Events_MonthMinOrderByAggregateInput, {nullable:true})
    _min?: Events_MonthMinOrderByAggregateInput;

    @Field(() => Events_MonthSumOrderByAggregateInput, {nullable:true})
    _sum?: Events_MonthSumOrderByAggregateInput;
}
