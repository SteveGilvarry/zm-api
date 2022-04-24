import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { BigIntWithAggregatesFilter } from '../prisma/big-int-with-aggregates-filter.input';

@InputType()
export class Snapshot_EventsScalarWhereWithAggregatesInput {

    @Field(() => [Snapshot_EventsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<Snapshot_EventsScalarWhereWithAggregatesInput>;

    @Field(() => [Snapshot_EventsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<Snapshot_EventsScalarWhereWithAggregatesInput>;

    @Field(() => [Snapshot_EventsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<Snapshot_EventsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    SnapshotId?: IntWithAggregatesFilter;

    @Field(() => BigIntWithAggregatesFilter, {nullable:true})
    EventId?: BigIntWithAggregatesFilter;
}
