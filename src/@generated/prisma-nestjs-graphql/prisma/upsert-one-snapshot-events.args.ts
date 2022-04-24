import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsWhereUniqueInput } from '../snapshot-events/snapshot-events-where-unique.input';
import { Snapshot_EventsCreateInput } from '../snapshot-events/snapshot-events-create.input';
import { Snapshot_EventsUpdateInput } from '../snapshot-events/snapshot-events-update.input';

@ArgsType()
export class UpsertOneSnapshotEventsArgs {

    @Field(() => Snapshot_EventsWhereUniqueInput, {nullable:false})
    where!: Snapshot_EventsWhereUniqueInput;

    @Field(() => Snapshot_EventsCreateInput, {nullable:false})
    create!: Snapshot_EventsCreateInput;

    @Field(() => Snapshot_EventsUpdateInput, {nullable:false})
    update!: Snapshot_EventsUpdateInput;
}
