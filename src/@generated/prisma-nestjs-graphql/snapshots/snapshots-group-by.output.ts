import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { SnapshotsCountAggregate } from './snapshots-count-aggregate.output';
import { SnapshotsAvgAggregate } from './snapshots-avg-aggregate.output';
import { SnapshotsSumAggregate } from './snapshots-sum-aggregate.output';
import { SnapshotsMinAggregate } from './snapshots-min-aggregate.output';
import { SnapshotsMaxAggregate } from './snapshots-max-aggregate.output';

@ObjectType()
export class SnapshotsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => String, {nullable:true})
    Description?: string;

    @Field(() => Int, {nullable:true})
    CreatedBy?: number;

    @Field(() => Date, {nullable:true})
    CreatedOn?: Date | string;

    @Field(() => SnapshotsCountAggregate, {nullable:true})
    _count?: SnapshotsCountAggregate;

    @Field(() => SnapshotsAvgAggregate, {nullable:true})
    _avg?: SnapshotsAvgAggregate;

    @Field(() => SnapshotsSumAggregate, {nullable:true})
    _sum?: SnapshotsSumAggregate;

    @Field(() => SnapshotsMinAggregate, {nullable:true})
    _min?: SnapshotsMinAggregate;

    @Field(() => SnapshotsMaxAggregate, {nullable:true})
    _max?: SnapshotsMaxAggregate;
}
