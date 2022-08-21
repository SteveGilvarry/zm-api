import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsWhereUniqueInput } from '../snapshot-events/snapshot-events-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneSnapshotEventsArgs {

    @Field(() => Snapshot_EventsWhereUniqueInput, {nullable:false})
    @Type(() => Snapshot_EventsWhereUniqueInput)
    where!: Snapshot_EventsWhereUniqueInput;
}
