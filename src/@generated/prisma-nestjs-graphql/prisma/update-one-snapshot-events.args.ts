import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsUpdateInput } from '../snapshot-events/snapshot-events-update.input';
import { Type } from 'class-transformer';
import { Snapshot_EventsWhereUniqueInput } from '../snapshot-events/snapshot-events-where-unique.input';

@ArgsType()
export class UpdateOneSnapshotEventsArgs {

    @Field(() => Snapshot_EventsUpdateInput, {nullable:false})
    @Type(() => Snapshot_EventsUpdateInput)
    data!: Snapshot_EventsUpdateInput;

    @Field(() => Snapshot_EventsWhereUniqueInput, {nullable:false})
    @Type(() => Snapshot_EventsWhereUniqueInput)
    where!: Snapshot_EventsWhereUniqueInput;
}
