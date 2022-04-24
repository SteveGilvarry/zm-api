import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Events_WeekCountAggregate } from './events-week-count-aggregate.output';
import { Events_WeekAvgAggregate } from './events-week-avg-aggregate.output';
import { Events_WeekSumAggregate } from './events-week-sum-aggregate.output';
import { Events_WeekMinAggregate } from './events-week-min-aggregate.output';
import { Events_WeekMaxAggregate } from './events-week-max-aggregate.output';

@ObjectType()
export class Events_WeekGroupBy {

    @Field(() => Int, {nullable:false})
    EventId!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Date, {nullable:true})
    StartDateTime?: Date | string;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;

    @Field(() => Events_WeekCountAggregate, {nullable:true})
    _count?: Events_WeekCountAggregate;

    @Field(() => Events_WeekAvgAggregate, {nullable:true})
    _avg?: Events_WeekAvgAggregate;

    @Field(() => Events_WeekSumAggregate, {nullable:true})
    _sum?: Events_WeekSumAggregate;

    @Field(() => Events_WeekMinAggregate, {nullable:true})
    _min?: Events_WeekMinAggregate;

    @Field(() => Events_WeekMaxAggregate, {nullable:true})
    _max?: Events_WeekMaxAggregate;
}
