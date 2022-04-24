import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsUpdateManyMutationInput } from '../snapshot-events/snapshot-events-update-many-mutation.input';
import { Snapshot_EventsWhereInput } from '../snapshot-events/snapshot-events-where.input';

@ArgsType()
export class UpdateManySnapshotEventsArgs {

    @Field(() => Snapshot_EventsUpdateManyMutationInput, {nullable:false})
    data!: Snapshot_EventsUpdateManyMutationInput;

    @Field(() => Snapshot_EventsWhereInput, {nullable:true})
    where?: Snapshot_EventsWhereInput;
}
