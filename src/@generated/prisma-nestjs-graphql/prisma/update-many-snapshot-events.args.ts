import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsUpdateManyMutationInput } from '../snapshot-events/snapshot-events-update-many-mutation.input';
import { Type } from 'class-transformer';
import { Snapshot_EventsWhereInput } from '../snapshot-events/snapshot-events-where.input';

@ArgsType()
export class UpdateManySnapshotEventsArgs {

    @Field(() => Snapshot_EventsUpdateManyMutationInput, {nullable:false})
    @Type(() => Snapshot_EventsUpdateManyMutationInput)
    data!: Snapshot_EventsUpdateManyMutationInput;

    @Field(() => Snapshot_EventsWhereInput, {nullable:true})
    @Type(() => Snapshot_EventsWhereInput)
    where?: Snapshot_EventsWhereInput;
}
