import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Events_DayCountAggregate } from './events-day-count-aggregate.output';
import { Events_DayAvgAggregate } from './events-day-avg-aggregate.output';
import { Events_DaySumAggregate } from './events-day-sum-aggregate.output';
import { Events_DayMinAggregate } from './events-day-min-aggregate.output';
import { Events_DayMaxAggregate } from './events-day-max-aggregate.output';

@ObjectType()
export class Events_DayGroupBy {

    @Field(() => Int, {nullable:false})
    EventId!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Date, {nullable:true})
    StartDateTime?: Date | string;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;

    @Field(() => Events_DayCountAggregate, {nullable:true})
    _count?: Events_DayCountAggregate;

    @Field(() => Events_DayAvgAggregate, {nullable:true})
    _avg?: Events_DayAvgAggregate;

    @Field(() => Events_DaySumAggregate, {nullable:true})
    _sum?: Events_DaySumAggregate;

    @Field(() => Events_DayMinAggregate, {nullable:true})
    _min?: Events_DayMinAggregate;

    @Field(() => Events_DayMaxAggregate, {nullable:true})
    _max?: Events_DayMaxAggregate;
}
