import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class Event_SummariesAvgAggregate {

    @Field(() => Float, {nullable:true})
    MonitorId?: number;

    @Field(() => Float, {nullable:true})
    TotalEvents?: number;

    @Field(() => Float, {nullable:true})
    TotalEventDiskSpace?: number;

    @Field(() => Float, {nullable:true})
    HourEvents?: number;

    @Field(() => Float, {nullable:true})
    HourEventDiskSpace?: number;

    @Field(() => Float, {nullable:true})
    DayEvents?: number;

    @Field(() => Float, {nullable:true})
    DayEventDiskSpace?: number;

    @Field(() => Float, {nullable:true})
    WeekEvents?: number;

    @Field(() => Float, {nullable:true})
    WeekEventDiskSpace?: number;

    @Field(() => Float, {nullable:true})
    MonthEvents?: number;

    @Field(() => Float, {nullable:true})
    MonthEventDiskSpace?: number;

    @Field(() => Float, {nullable:true})
    ArchivedEvents?: number;

    @Field(() => Float, {nullable:true})
    ArchivedEventDiskSpace?: number;
}
