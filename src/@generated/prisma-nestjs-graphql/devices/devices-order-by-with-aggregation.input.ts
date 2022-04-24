import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { DevicesCountOrderByAggregateInput } from './devices-count-order-by-aggregate.input';
import { DevicesAvgOrderByAggregateInput } from './devices-avg-order-by-aggregate.input';
import { DevicesMaxOrderByAggregateInput } from './devices-max-order-by-aggregate.input';
import { DevicesMinOrderByAggregateInput } from './devices-min-order-by-aggregate.input';
import { DevicesSumOrderByAggregateInput } from './devices-sum-order-by-aggregate.input';

@InputType()
export class DevicesOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    KeyString?: keyof typeof SortOrder;

    @Field(() => DevicesCountOrderByAggregateInput, {nullable:true})
    _count?: DevicesCountOrderByAggregateInput;

    @Field(() => DevicesAvgOrderByAggregateInput, {nullable:true})
    _avg?: DevicesAvgOrderByAggregateInput;

    @Field(() => DevicesMaxOrderByAggregateInput, {nullable:true})
    _max?: DevicesMaxOrderByAggregateInput;

    @Field(() => DevicesMinOrderByAggregateInput, {nullable:true})
    _min?: DevicesMinOrderByAggregateInput;

    @Field(() => DevicesSumOrderByAggregateInput, {nullable:true})
    _sum?: DevicesSumOrderByAggregateInput;
}
