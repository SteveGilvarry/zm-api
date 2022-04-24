import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Events_HourCountAggregate } from './events-hour-count-aggregate.output';
import { Events_HourAvgAggregate } from './events-hour-avg-aggregate.output';
import { Events_HourSumAggregate } from './events-hour-sum-aggregate.output';
import { Events_HourMinAggregate } from './events-hour-min-aggregate.output';
import { Events_HourMaxAggregate } from './events-hour-max-aggregate.output';

@ObjectType()
export class AggregateEvents_Hour {

    @Field(() => Events_HourCountAggregate, {nullable:true})
    _count?: Events_HourCountAggregate;

    @Field(() => Events_HourAvgAggregate, {nullable:true})
    _avg?: Events_HourAvgAggregate;

    @Field(() => Events_HourSumAggregate, {nullable:true})
    _sum?: Events_HourSumAggregate;

    @Field(() => Events_HourMinAggregate, {nullable:true})
    _min?: Events_HourMinAggregate;

    @Field(() => Events_HourMaxAggregate, {nullable:true})
    _max?: Events_HourMaxAggregate;
}
