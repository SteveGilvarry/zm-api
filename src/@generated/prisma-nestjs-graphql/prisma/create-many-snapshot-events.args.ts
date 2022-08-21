import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsCreateManyInput } from '../snapshot-events/snapshot-events-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManySnapshotEventsArgs {

    @Field(() => [Snapshot_EventsCreateManyInput], {nullable:false})
    @Type(() => Snapshot_EventsCreateManyInput)
    data!: Array<Snapshot_EventsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
