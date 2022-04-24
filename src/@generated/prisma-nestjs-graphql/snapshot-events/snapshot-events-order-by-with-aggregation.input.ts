import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Snapshot_EventsCountOrderByAggregateInput } from './snapshot-events-count-order-by-aggregate.input';
import { Snapshot_EventsAvgOrderByAggregateInput } from './snapshot-events-avg-order-by-aggregate.input';
import { Snapshot_EventsMaxOrderByAggregateInput } from './snapshot-events-max-order-by-aggregate.input';
import { Snapshot_EventsMinOrderByAggregateInput } from './snapshot-events-min-order-by-aggregate.input';
import { Snapshot_EventsSumOrderByAggregateInput } from './snapshot-events-sum-order-by-aggregate.input';

@InputType()
export class Snapshot_EventsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SnapshotId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => Snapshot_EventsCountOrderByAggregateInput, {nullable:true})
    _count?: Snapshot_EventsCountOrderByAggregateInput;

    @Field(() => Snapshot_EventsAvgOrderByAggregateInput, {nullable:true})
    _avg?: Snapshot_EventsAvgOrderByAggregateInput;

    @Field(() => Snapshot_EventsMaxOrderByAggregateInput, {nullable:true})
    _max?: Snapshot_EventsMaxOrderByAggregateInput;

    @Field(() => Snapshot_EventsMinOrderByAggregateInput, {nullable:true})
    _min?: Snapshot_EventsMinOrderByAggregateInput;

    @Field(() => Snapshot_EventsSumOrderByAggregateInput, {nullable:true})
    _sum?: Snapshot_EventsSumOrderByAggregateInput;
}
