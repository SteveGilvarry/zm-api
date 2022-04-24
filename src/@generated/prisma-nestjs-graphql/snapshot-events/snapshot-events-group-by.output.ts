import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Snapshot_EventsCountAggregate } from './snapshot-events-count-aggregate.output';
import { Snapshot_EventsAvgAggregate } from './snapshot-events-avg-aggregate.output';
import { Snapshot_EventsSumAggregate } from './snapshot-events-sum-aggregate.output';
import { Snapshot_EventsMinAggregate } from './snapshot-events-min-aggregate.output';
import { Snapshot_EventsMaxAggregate } from './snapshot-events-max-aggregate.output';

@ObjectType()
export class Snapshot_EventsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    SnapshotId!: number;

    @Field(() => String, {nullable:false})
    EventId!: bigint | number;

    @Field(() => Snapshot_EventsCountAggregate, {nullable:true})
    _count?: Snapshot_EventsCountAggregate;

    @Field(() => Snapshot_EventsAvgAggregate, {nullable:true})
    _avg?: Snapshot_EventsAvgAggregate;

    @Field(() => Snapshot_EventsSumAggregate, {nullable:true})
    _sum?: Snapshot_EventsSumAggregate;

    @Field(() => Snapshot_EventsMinAggregate, {nullable:true})
    _min?: Snapshot_EventsMinAggregate;

    @Field(() => Snapshot_EventsMaxAggregate, {nullable:true})
    _max?: Snapshot_EventsMaxAggregate;
}
