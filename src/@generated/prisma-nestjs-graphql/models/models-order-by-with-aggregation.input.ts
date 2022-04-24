import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { ModelsCountOrderByAggregateInput } from './models-count-order-by-aggregate.input';
import { ModelsAvgOrderByAggregateInput } from './models-avg-order-by-aggregate.input';
import { ModelsMaxOrderByAggregateInput } from './models-max-order-by-aggregate.input';
import { ModelsMinOrderByAggregateInput } from './models-min-order-by-aggregate.input';
import { ModelsSumOrderByAggregateInput } from './models-sum-order-by-aggregate.input';

@InputType()
export class ModelsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ManufacturerId?: keyof typeof SortOrder;

    @Field(() => ModelsCountOrderByAggregateInput, {nullable:true})
    _count?: ModelsCountOrderByAggregateInput;

    @Field(() => ModelsAvgOrderByAggregateInput, {nullable:true})
    _avg?: ModelsAvgOrderByAggregateInput;

    @Field(() => ModelsMaxOrderByAggregateInput, {nullable:true})
    _max?: ModelsMaxOrderByAggregateInput;

    @Field(() => ModelsMinOrderByAggregateInput, {nullable:true})
    _min?: ModelsMinOrderByAggregateInput;

    @Field(() => ModelsSumOrderByAggregateInput, {nullable:true})
    _sum?: ModelsSumOrderByAggregateInput;
}
