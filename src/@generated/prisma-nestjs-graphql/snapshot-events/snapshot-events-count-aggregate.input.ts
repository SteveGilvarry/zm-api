import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class Snapshot_EventsCountAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    SnapshotId?: true;

    @Field(() => Boolean, {nullable:true})
    EventId?: true;

    @Field(() => Boolean, {nullable:true})
    _all?: true;
}
