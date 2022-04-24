import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { StorageCountOrderByAggregateInput } from './storage-count-order-by-aggregate.input';
import { StorageAvgOrderByAggregateInput } from './storage-avg-order-by-aggregate.input';
import { StorageMaxOrderByAggregateInput } from './storage-max-order-by-aggregate.input';
import { StorageMinOrderByAggregateInput } from './storage-min-order-by-aggregate.input';
import { StorageSumOrderByAggregateInput } from './storage-sum-order-by-aggregate.input';

@InputType()
export class StorageOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Path?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Url?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Scheme?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ServerId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DoDelete?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Enabled?: keyof typeof SortOrder;

    @Field(() => StorageCountOrderByAggregateInput, {nullable:true})
    _count?: StorageCountOrderByAggregateInput;

    @Field(() => StorageAvgOrderByAggregateInput, {nullable:true})
    _avg?: StorageAvgOrderByAggregateInput;

    @Field(() => StorageMaxOrderByAggregateInput, {nullable:true})
    _max?: StorageMaxOrderByAggregateInput;

    @Field(() => StorageMinOrderByAggregateInput, {nullable:true})
    _min?: StorageMinOrderByAggregateInput;

    @Field(() => StorageSumOrderByAggregateInput, {nullable:true})
    _sum?: StorageSumOrderByAggregateInput;
}
