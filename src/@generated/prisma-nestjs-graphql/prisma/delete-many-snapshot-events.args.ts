import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsWhereInput } from '../snapshot-events/snapshot-events-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManySnapshotEventsArgs {

    @Field(() => Snapshot_EventsWhereInput, {nullable:true})
    @Type(() => Snapshot_EventsWhereInput)
    where?: Snapshot_EventsWhereInput;
}
