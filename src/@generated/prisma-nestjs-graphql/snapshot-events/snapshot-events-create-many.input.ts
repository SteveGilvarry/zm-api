import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class Snapshot_EventsCreateManyInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:false})
    SnapshotId!: number;

    @Field(() => String, {nullable:false})
    EventId!: bigint | number;
}
