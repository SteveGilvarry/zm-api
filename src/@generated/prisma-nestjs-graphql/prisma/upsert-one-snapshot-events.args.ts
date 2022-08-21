import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsWhereUniqueInput } from '../snapshot-events/snapshot-events-where-unique.input';
import { Type } from 'class-transformer';
import { Snapshot_EventsCreateInput } from '../snapshot-events/snapshot-events-create.input';
import { Snapshot_EventsUpdateInput } from '../snapshot-events/snapshot-events-update.input';

@ArgsType()
export class UpsertOneSnapshotEventsArgs {

    @Field(() => Snapshot_EventsWhereUniqueInput, {nullable:false})
    @Type(() => Snapshot_EventsWhereUniqueInput)
    where!: Snapshot_EventsWhereUniqueInput;

    @Field(() => Snapshot_EventsCreateInput, {nullable:false})
    @Type(() => Snapshot_EventsCreateInput)
    create!: Snapshot_EventsCreateInput;

    @Field(() => Snapshot_EventsUpdateInput, {nullable:false})
    @Type(() => Snapshot_EventsUpdateInput)
    update!: Snapshot_EventsUpdateInput;
}
