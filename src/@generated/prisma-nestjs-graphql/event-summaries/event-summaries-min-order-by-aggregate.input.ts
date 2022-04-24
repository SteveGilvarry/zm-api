import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class Event_SummariesMinOrderByAggregateInput {

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
}
