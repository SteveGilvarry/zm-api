import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereInput } from './storage-where.input';
import { StorageOrderByWithRelationInput } from './storage-order-by-with-relation.input';
import { StorageWhereUniqueInput } from './storage-where-unique.input';
import { Int } from '@nestjs/graphql';
import { StorageCountAggregateInput } from './storage-count-aggregate.input';
import { StorageAvgAggregateInput } from './storage-avg-aggregate.input';
import { StorageSumAggregateInput } from './storage-sum-aggregate.input';
import { StorageMinAggregateInput } from './storage-min-aggregate.input';
import { StorageMaxAggregateInput } from './storage-max-aggregate.input';

@ArgsType()
export class StorageAggregateArgs {

    @Field(() => StorageWhereInput, {nullable:true})
    where?: StorageWhereInput;

    @Field(() => [StorageOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<StorageOrderByWithRelationInput>;

    @Field(() => StorageWhereUniqueInput, {nullable:true})
    cursor?: StorageWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => StorageCountAggregateInput, {nullable:true})
    _count?: StorageCountAggregateInput;

    @Field(() => StorageAvgAggregateInput, {nullable:true})
    _avg?: StorageAvgAggregateInput;

    @Field(() => StorageSumAggregateInput, {nullable:true})
    _sum?: StorageSumAggregateInput;

    @Field(() => StorageMinAggregateInput, {nullable:true})
    _min?: StorageMinAggregateInput;

    @Field(() => StorageMaxAggregateInput, {nullable:true})
    _max?: StorageMaxAggregateInput;
}
