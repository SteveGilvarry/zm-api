import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Events_ArchivedCountAggregate } from './events-archived-count-aggregate.output';
import { Events_ArchivedAvgAggregate } from './events-archived-avg-aggregate.output';
import { Events_ArchivedSumAggregate } from './events-archived-sum-aggregate.output';
import { Events_ArchivedMinAggregate } from './events-archived-min-aggregate.output';
import { Events_ArchivedMaxAggregate } from './events-archived-max-aggregate.output';

@ObjectType()
export class AggregateEvents_Archived {

    @Field(() => Events_ArchivedCountAggregate, {nullable:true})
    _count?: Events_ArchivedCountAggregate;

    @Field(() => Events_ArchivedAvgAggregate, {nullable:true})
    _avg?: Events_ArchivedAvgAggregate;

    @Field(() => Events_ArchivedSumAggregate, {nullable:true})
    _sum?: Events_ArchivedSumAggregate;

    @Field(() => Events_ArchivedMinAggregate, {nullable:true})
    _min?: Events_ArchivedMinAggregate;

    @Field(() => Events_ArchivedMaxAggregate, {nullable:true})
    _max?: Events_ArchivedMaxAggregate;
}
