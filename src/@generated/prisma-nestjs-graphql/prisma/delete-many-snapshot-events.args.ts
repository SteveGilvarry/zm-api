import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsWhereInput } from '../snapshot-events/snapshot-events-where.input';

@ArgsType()
export class DeleteManySnapshotEventsArgs {

    @Field(() => Snapshot_EventsWhereInput, {nullable:true})
    where?: Snapshot_EventsWhereInput;
}
