import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Snapshot_EventsSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    SnapshotId?: number;

    @Field(() => String, {nullable:true})
    EventId?: bigint | number;
}
