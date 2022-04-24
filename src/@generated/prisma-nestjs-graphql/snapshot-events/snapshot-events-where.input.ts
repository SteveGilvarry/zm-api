import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { BigIntFilter } from '../prisma/big-int-filter.input';

@InputType()
export class Snapshot_EventsWhereInput {

    @Field(() => [Snapshot_EventsWhereInput], {nullable:true})
    AND?: Array<Snapshot_EventsWhereInput>;

    @Field(() => [Snapshot_EventsWhereInput], {nullable:true})
    OR?: Array<Snapshot_EventsWhereInput>;

    @Field(() => [Snapshot_EventsWhereInput], {nullable:true})
    NOT?: Array<Snapshot_EventsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    SnapshotId?: IntFilter;

    @Field(() => BigIntFilter, {nullable:true})
    EventId?: BigIntFilter;
}
