import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereInput } from './snapshots-where.input';
import { Type } from 'class-transformer';
import { SnapshotsOrderByWithAggregationInput } from './snapshots-order-by-with-aggregation.input';
import { SnapshotsScalarFieldEnum } from './snapshots-scalar-field.enum';
import { SnapshotsScalarWhereWithAggregatesInput } from './snapshots-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { SnapshotsCountAggregateInput } from './snapshots-count-aggregate.input';
import { SnapshotsAvgAggregateInput } from './snapshots-avg-aggregate.input';
import { SnapshotsSumAggregateInput } from './snapshots-sum-aggregate.input';
import { SnapshotsMinAggregateInput } from './snapshots-min-aggregate.input';
import { SnapshotsMaxAggregateInput } from './snapshots-max-aggregate.input';

@ArgsType()
export class SnapshotsGroupByArgs {

    @Field(() => SnapshotsWhereInput, {nullable:true})
    @Type(() => SnapshotsWhereInput)
    where?: SnapshotsWhereInput;

    @Field(() => [SnapshotsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<SnapshotsOrderByWithAggregationInput>;

    @Field(() => [SnapshotsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof SnapshotsScalarFieldEnum>;

    @Field(() => SnapshotsScalarWhereWithAggregatesInput, {nullable:true})
    having?: SnapshotsScalarWhereWithAggregatesInput;

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
