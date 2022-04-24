import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsWhereInput } from '../snapshot-events/snapshot-events-where.input';
import { Snapshot_EventsOrderByWithAggregationInput } from '../snapshot-events/snapshot-events-order-by-with-aggregation.input';
import { Snapshot_EventsScalarFieldEnum } from '../snapshot-events/snapshot-events-scalar-field.enum';
import { Snapshot_EventsScalarWhereWithAggregatesInput } from '../snapshot-events/snapshot-events-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupBySnapshotEventsArgs {

    @Field(() => Snapshot_EventsWhereInput, {nullable:true})
    where?: Snapshot_EventsWhereInput;

    @Field(() => [Snapshot_EventsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Snapshot_EventsOrderByWithAggregationInput>;

    @Field(() => [Snapshot_EventsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Snapshot_EventsScalarFieldEnum>;

    @Field(() => Snapshot_EventsScalarWhereWithAggregatesInput, {nullable:true})
    having?: Snapshot_EventsScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
