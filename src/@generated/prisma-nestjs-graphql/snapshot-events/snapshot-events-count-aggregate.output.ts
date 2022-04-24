import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Snapshot_EventsCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    SnapshotId!: number;

    @Field(() => Int, {nullable:false})
    EventId!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
