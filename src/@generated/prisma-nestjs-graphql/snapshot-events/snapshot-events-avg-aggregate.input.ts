import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class Snapshot_EventsAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    SnapshotId?: true;

    @Field(() => Boolean, {nullable:true})
    EventId?: true;
}
