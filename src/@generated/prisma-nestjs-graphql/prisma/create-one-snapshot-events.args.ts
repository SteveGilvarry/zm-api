import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsCreateInput } from '../snapshot-events/snapshot-events-create.input';

@ArgsType()
export class CreateOneSnapshotEventsArgs {

    @Field(() => Snapshot_EventsCreateInput, {nullable:false})
    data!: Snapshot_EventsCreateInput;
}
