import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { MontageLayoutsCountOrderByAggregateInput } from './montage-layouts-count-order-by-aggregate.input';
import { MontageLayoutsAvgOrderByAggregateInput } from './montage-layouts-avg-order-by-aggregate.input';
import { MontageLayoutsMaxOrderByAggregateInput } from './montage-layouts-max-order-by-aggregate.input';
import { MontageLayoutsMinOrderByAggregateInput } from './montage-layouts-min-order-by-aggregate.input';
import { MontageLayoutsSumOrderByAggregateInput } from './montage-layouts-sum-order-by-aggregate.input';

@InputType()
export class MontageLayoutsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Positions?: keyof typeof SortOrder;

    @Field(() => MontageLayoutsCountOrderByAggregateInput, {nullable:true})
    _count?: MontageLayoutsCountOrderByAggregateInput;

    @Field(() => MontageLayoutsAvgOrderByAggregateInput, {nullable:true})
    _avg?: MontageLayoutsAvgOrderByAggregateInput;

    @Field(() => MontageLayoutsMaxOrderByAggregateInput, {nullable:true})
    _max?: MontageLayoutsMaxOrderByAggregateInput;

    @Field(() => MontageLayoutsMinOrderByAggregateInput, {nullable:true})
    _min?: MontageLayoutsMinOrderByAggregateInput;

    @Field(() => MontageLayoutsSumOrderByAggregateInput, {nullable:true})
    _sum?: MontageLayoutsSumOrderByAggregateInput;
}
