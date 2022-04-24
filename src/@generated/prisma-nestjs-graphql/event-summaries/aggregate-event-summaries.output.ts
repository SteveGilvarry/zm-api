import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Event_SummariesCountAggregate } from './event-summaries-count-aggregate.output';
import { Event_SummariesAvgAggregate } from './event-summaries-avg-aggregate.output';
import { Event_SummariesSumAggregate } from './event-summaries-sum-aggregate.output';
import { Event_SummariesMinAggregate } from './event-summaries-min-aggregate.output';
import { Event_SummariesMaxAggregate } from './event-summaries-max-aggregate.output';

@ObjectType()
export class AggregateEvent_Summaries {

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
