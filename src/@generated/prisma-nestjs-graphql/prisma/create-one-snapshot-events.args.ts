import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsCreateInput } from '../snapshot-events/snapshot-events-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneSnapshotEventsArgs {

    @Field(() => Snapshot_EventsCreateInput, {nullable:false})
    @Type(() => Snapshot_EventsCreateInput)
    data!: Snapshot_EventsCreateInput;
}
