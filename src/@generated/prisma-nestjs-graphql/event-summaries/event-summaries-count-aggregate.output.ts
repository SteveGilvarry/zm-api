import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Event_SummariesCountAggregate {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    TotalEvents!: number;

    @Field(() => Int, {nullable:false})
    TotalEventDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    HourEvents!: number;

    @Field(() => Int, {nullable:false})
    HourEventDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    DayEvents!: number;

    @Field(() => Int, {nullable:false})
    DayEventDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    WeekEvents!: number;

    @Field(() => Int, {nullable:false})
    WeekEventDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    MonthEvents!: number;

    @Field(() => Int, {nullable:false})
    MonthEventDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    ArchivedEvents!: number;

    @Field(() => Int, {nullable:false})
    ArchivedEventDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
