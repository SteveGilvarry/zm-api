import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { IntNullableWithAggregatesFilter } from '../prisma/int-nullable-with-aggregates-filter.input';
import { BigIntNullableWithAggregatesFilter } from '../prisma/big-int-nullable-with-aggregates-filter.input';

@InputType()
export class Event_SummariesScalarWhereWithAggregatesInput {

    @Field(() => [Event_SummariesScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<Event_SummariesScalarWhereWithAggregatesInput>;

    @Field(() => [Event_SummariesScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<Event_SummariesScalarWhereWithAggregatesInput>;

    @Field(() => [Event_SummariesScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<Event_SummariesScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    TotalEvents?: IntNullableWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    TotalEventDiskSpace?: BigIntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    HourEvents?: IntNullableWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    HourEventDiskSpace?: BigIntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    DayEvents?: IntNullableWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    DayEventDiskSpace?: BigIntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    WeekEvents?: IntNullableWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    WeekEventDiskSpace?: BigIntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MonthEvents?: IntNullableWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    MonthEventDiskSpace?: BigIntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    ArchivedEvents?: IntNullableWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    ArchivedEventDiskSpace?: BigIntNullableWithAggregatesFilter;
}
