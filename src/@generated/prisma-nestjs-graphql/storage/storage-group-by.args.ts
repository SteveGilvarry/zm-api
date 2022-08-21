import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereInput } from './storage-where.input';
import { Type } from 'class-transformer';
import { StorageOrderByWithAggregationInput } from './storage-order-by-with-aggregation.input';
import { StorageScalarFieldEnum } from './storage-scalar-field.enum';
import { StorageScalarWhereWithAggregatesInput } from './storage-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { StorageCountAggregateInput } from './storage-count-aggregate.input';
import { StorageAvgAggregateInput } from './storage-avg-aggregate.input';
import { StorageSumAggregateInput } from './storage-sum-aggregate.input';
import { StorageMinAggregateInput } from './storage-min-aggregate.input';
import { StorageMaxAggregateInput } from './storage-max-aggregate.input';

@ArgsType()
export class StorageGroupByArgs {

    @Field(() => StorageWhereInput, {nullable:true})
    @Type(() => StorageWhereInput)
    where?: StorageWhereInput;

    @Field(() => [StorageOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<StorageOrderByWithAggregationInput>;

    @Field(() => [StorageScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof StorageScalarFieldEnum>;

    @Field(() => StorageScalarWhereWithAggregatesInput, {nullable:true})
    having?: StorageScalarWhereWithAggregatesInput;

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
