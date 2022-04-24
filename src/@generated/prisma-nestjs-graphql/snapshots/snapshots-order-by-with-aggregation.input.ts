import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { SnapshotsCountOrderByAggregateInput } from './snapshots-count-order-by-aggregate.input';
import { SnapshotsAvgOrderByAggregateInput } from './snapshots-avg-order-by-aggregate.input';
import { SnapshotsMaxOrderByAggregateInput } from './snapshots-max-order-by-aggregate.input';
import { SnapshotsMinOrderByAggregateInput } from './snapshots-min-order-by-aggregate.input';
import { SnapshotsSumOrderByAggregateInput } from './snapshots-sum-order-by-aggregate.input';

@InputType()
export class SnapshotsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Description?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CreatedBy?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CreatedOn?: keyof typeof SortOrder;

    @Field(() => SnapshotsCountOrderByAggregateInput, {nullable:true})
    _count?: SnapshotsCountOrderByAggregateInput;

    @Field(() => SnapshotsAvgOrderByAggregateInput, {nullable:true})
    _avg?: SnapshotsAvgOrderByAggregateInput;

    @Field(() => SnapshotsMaxOrderByAggregateInput, {nullable:true})
    _max?: SnapshotsMaxOrderByAggregateInput;

    @Field(() => SnapshotsMinOrderByAggregateInput, {nullable:true})
    _min?: SnapshotsMinOrderByAggregateInput;

    @Field(() => SnapshotsSumOrderByAggregateInput, {nullable:true})
    _sum?: SnapshotsSumOrderByAggregateInput;
}
