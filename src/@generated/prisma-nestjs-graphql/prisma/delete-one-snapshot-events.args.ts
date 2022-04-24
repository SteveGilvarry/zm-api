import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsWhereUniqueInput } from '../snapshot-events/snapshot-events-where-unique.input';

@ArgsType()
export class DeleteOneSnapshotEventsArgs {

    @Field(() => Snapshot_EventsWhereUniqueInput, {nullable:false})
    where!: Snapshot_EventsWhereUniqueInput;
}
