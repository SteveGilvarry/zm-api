import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Events_MonthCountAggregate } from './events-month-count-aggregate.output';
import { Events_MonthAvgAggregate } from './events-month-avg-aggregate.output';
import { Events_MonthSumAggregate } from './events-month-sum-aggregate.output';
import { Events_MonthMinAggregate } from './events-month-min-aggregate.output';
import { Events_MonthMaxAggregate } from './events-month-max-aggregate.output';

@ObjectType()
export class AggregateEvents_Month {

    @Field(() => Events_MonthCountAggregate, {nullable:true})
    _count?: Events_MonthCountAggregate;

    @Field(() => Events_MonthAvgAggregate, {nullable:true})
    _avg?: Events_MonthAvgAggregate;

    @Field(() => Events_MonthSumAggregate, {nullable:true})
    _sum?: Events_MonthSumAggregate;

    @Field(() => Events_MonthMinAggregate, {nullable:true})
    _min?: Events_MonthMinAggregate;

    @Field(() => Events_MonthMaxAggregate, {nullable:true})
    _max?: Events_MonthMaxAggregate;
}
