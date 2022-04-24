import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Event_SummariesMaxAggregate {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => Int, {nullable:true})
    TotalEvents?: number;

    @Field(() => String, {nullable:true})
    TotalEventDiskSpace?: bigint | number;

    @Field(() => Int, {nullable:true})
    HourEvents?: number;

    @Field(() => String, {nullable:true})
    HourEventDiskSpace?: bigint | number;

    @Field(() => Int, {nullable:true})
    DayEvents?: number;

    @Field(() => String, {nullable:true})
    DayEventDiskSpace?: bigint | number;

    @Field(() => Int, {nullable:true})
    WeekEvents?: number;

    @Field(() => String, {nullable:true})
    WeekEventDiskSpace?: bigint | number;

    @Field(() => Int, {nullable:true})
    MonthEvents?: number;

    @Field(() => String, {nullable:true})
    MonthEventDiskSpace?: bigint | number;

    @Field(() => Int, {nullable:true})
    ArchivedEvents?: number;

    @Field(() => String, {nullable:true})
    ArchivedEventDiskSpace?: bigint | number;
}
