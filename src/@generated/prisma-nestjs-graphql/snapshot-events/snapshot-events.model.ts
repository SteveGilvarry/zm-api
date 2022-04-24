import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Snapshot_Events {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    SnapshotId!: number;

    @Field(() => String, {nullable:false})
    EventId!: bigint;
}
