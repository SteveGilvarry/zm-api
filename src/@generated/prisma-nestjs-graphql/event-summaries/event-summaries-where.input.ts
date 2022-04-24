import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { BigIntNullableFilter } from '../prisma/big-int-nullable-filter.input';

@InputType()
export class Event_SummariesWhereInput {

    @Field(() => [Event_SummariesWhereInput], {nullable:true})
    AND?: Array<Event_SummariesWhereInput>;

    @Field(() => [Event_SummariesWhereInput], {nullable:true})
    OR?: Array<Event_SummariesWhereInput>;

    @Field(() => [Event_SummariesWhereInput], {nullable:true})
    NOT?: Array<Event_SummariesWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    TotalEvents?: IntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    TotalEventDiskSpace?: BigIntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    HourEvents?: IntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    HourEventDiskSpace?: BigIntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    DayEvents?: IntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    DayEventDiskSpace?: BigIntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    WeekEvents?: IntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    WeekEventDiskSpace?: BigIntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MonthEvents?: IntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    MonthEventDiskSpace?: BigIntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    ArchivedEvents?: IntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    ArchivedEventDiskSpace?: BigIntNullableFilter;
}
