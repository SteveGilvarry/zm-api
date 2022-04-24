import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { ManufacturersCountOrderByAggregateInput } from './manufacturers-count-order-by-aggregate.input';
import { ManufacturersAvgOrderByAggregateInput } from './manufacturers-avg-order-by-aggregate.input';
import { ManufacturersMaxOrderByAggregateInput } from './manufacturers-max-order-by-aggregate.input';
import { ManufacturersMinOrderByAggregateInput } from './manufacturers-min-order-by-aggregate.input';
import { ManufacturersSumOrderByAggregateInput } from './manufacturers-sum-order-by-aggregate.input';

@InputType()
export class ManufacturersOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => ManufacturersCountOrderByAggregateInput, {nullable:true})
    _count?: ManufacturersCountOrderByAggregateInput;

    @Field(() => ManufacturersAvgOrderByAggregateInput, {nullable:true})
    _avg?: ManufacturersAvgOrderByAggregateInput;

    @Field(() => ManufacturersMaxOrderByAggregateInput, {nullable:true})
    _max?: ManufacturersMaxOrderByAggregateInput;

    @Field(() => ManufacturersMinOrderByAggregateInput, {nullable:true})
    _min?: ManufacturersMinOrderByAggregateInput;

    @Field(() => ManufacturersSumOrderByAggregateInput, {nullable:true})
    _sum?: ManufacturersSumOrderByAggregateInput;
}
