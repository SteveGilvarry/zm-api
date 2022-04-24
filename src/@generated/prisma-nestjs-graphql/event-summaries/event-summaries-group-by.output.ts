import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Event_SummariesCountAggregate } from './event-summaries-count-aggregate.output';
import { Event_SummariesAvgAggregate } from './event-summaries-avg-aggregate.output';
import { Event_SummariesSumAggregate } from './event-summaries-sum-aggregate.output';
import { Event_SummariesMinAggregate } from './event-summaries-min-aggregate.output';
import { Event_SummariesMaxAggregate } from './event-summaries-max-aggregate.output';

@ObjectType()
export class Event_SummariesGroupBy {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

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

    @Field(() => Event_SummariesCountAggregate, {nullable:true})
    _count?: Event_SummariesCountAggregate;

    @Field(() => Event_SummariesAvgAggregate, {nullable:true})
    _avg?: Event_SummariesAvgAggregate;

    @Field(() => Event_SummariesSumAggregate, {nullable:true})
    _sum?: Event_SummariesSumAggregate;

    @Field(() => Event_SummariesMinAggregate, {nullable:true})
    _min?: Event_SummariesMinAggregate;

    @Field(() => Event_SummariesMaxAggregate, {nullable:true})
    _max?: Event_SummariesMaxAggregate;
}
