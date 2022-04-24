import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Events_DayCountOrderByAggregateInput } from './events-day-count-order-by-aggregate.input';
import { Events_DayAvgOrderByAggregateInput } from './events-day-avg-order-by-aggregate.input';
import { Events_DayMaxOrderByAggregateInput } from './events-day-max-order-by-aggregate.input';
import { Events_DayMinOrderByAggregateInput } from './events-day-min-order-by-aggregate.input';
import { Events_DaySumOrderByAggregateInput } from './events-day-sum-order-by-aggregate.input';

@InputType()
export class Events_DayOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StartDateTime?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;

    @Field(() => Events_DayCountOrderByAggregateInput, {nullable:true})
    _count?: Events_DayCountOrderByAggregateInput;

    @Field(() => Events_DayAvgOrderByAggregateInput, {nullable:true})
    _avg?: Events_DayAvgOrderByAggregateInput;

    @Field(() => Events_DayMaxOrderByAggregateInput, {nullable:true})
    _max?: Events_DayMaxOrderByAggregateInput;

    @Field(() => Events_DayMinOrderByAggregateInput, {nullable:true})
    _min?: Events_DayMinOrderByAggregateInput;

    @Field(() => Events_DaySumOrderByAggregateInput, {nullable:true})
    _sum?: Events_DaySumOrderByAggregateInput;
}
