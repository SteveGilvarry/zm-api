import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Event_SummariesCountOrderByAggregateInput } from './event-summaries-count-order-by-aggregate.input';
import { Event_SummariesAvgOrderByAggregateInput } from './event-summaries-avg-order-by-aggregate.input';
import { Event_SummariesMaxOrderByAggregateInput } from './event-summaries-max-order-by-aggregate.input';
import { Event_SummariesMinOrderByAggregateInput } from './event-summaries-min-order-by-aggregate.input';
import { Event_SummariesSumOrderByAggregateInput } from './event-summaries-sum-order-by-aggregate.input';

@InputType()
export class Event_SummariesOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotalEvents?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotalEventDiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HourEvents?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HourEventDiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DayEvents?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DayEventDiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    WeekEvents?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    WeekEventDiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonthEvents?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonthEventDiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ArchivedEvents?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ArchivedEventDiskSpace?: keyof typeof SortOrder;

    @Field(() => Event_SummariesCountOrderByAggregateInput, {nullable:true})
    _count?: Event_SummariesCountOrderByAggregateInput;

    @Field(() => Event_SummariesAvgOrderByAggregateInput, {nullable:true})
    _avg?: Event_SummariesAvgOrderByAggregateInput;

    @Field(() => Event_SummariesMaxOrderByAggregateInput, {nullable:true})
    _max?: Event_SummariesMaxOrderByAggregateInput;

    @Field(() => Event_SummariesMinOrderByAggregateInput, {nullable:true})
    _min?: Event_SummariesMinOrderByAggregateInput;

    @Field(() => Event_SummariesSumOrderByAggregateInput, {nullable:true})
    _sum?: Event_SummariesSumOrderByAggregateInput;
}
