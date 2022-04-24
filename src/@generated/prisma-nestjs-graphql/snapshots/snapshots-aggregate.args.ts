import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereInput } from './snapshots-where.input';
import { SnapshotsOrderByWithRelationInput } from './snapshots-order-by-with-relation.input';
import { SnapshotsWhereUniqueInput } from './snapshots-where-unique.input';
import { Int } from '@nestjs/graphql';
import { SnapshotsCountAggregateInput } from './snapshots-count-aggregate.input';
import { SnapshotsAvgAggregateInput } from './snapshots-avg-aggregate.input';
import { SnapshotsSumAggregateInput } from './snapshots-sum-aggregate.input';
import { SnapshotsMinAggregateInput } from './snapshots-min-aggregate.input';
import { SnapshotsMaxAggregateInput } from './snapshots-max-aggregate.input';

@ArgsType()
export class SnapshotsAggregateArgs {

    @Field(() => SnapshotsWhereInput, {nullable:true})
    where?: SnapshotsWhereInput;

    @Field(() => [SnapshotsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<SnapshotsOrderByWithRelationInput>;

    @Field(() => SnapshotsWhereUniqueInput, {nullable:true})
    cursor?: SnapshotsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => SnapshotsCountAggregateInput, {nullable:true})
    _count?: SnapshotsCountAggregateInput;

    @Field(() => SnapshotsAvgAggregateInput, {nullable:true})
    _avg?: SnapshotsAvgAggregateInput;

    @Field(() => SnapshotsSumAggregateInput, {nullable:true})
    _sum?: SnapshotsSumAggregateInput;

    @Field(() => SnapshotsMinAggregateInput, {nullable:true})
    _min?: SnapshotsMinAggregateInput;

    @Field(() => SnapshotsMaxAggregateInput, {nullable:true})
    _max?: SnapshotsMaxAggregateInput;
}
