import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Groups_MonitorsCountOrderByAggregateInput } from './groups-monitors-count-order-by-aggregate.input';
import { Groups_MonitorsAvgOrderByAggregateInput } from './groups-monitors-avg-order-by-aggregate.input';
import { Groups_MonitorsMaxOrderByAggregateInput } from './groups-monitors-max-order-by-aggregate.input';
import { Groups_MonitorsMinOrderByAggregateInput } from './groups-monitors-min-order-by-aggregate.input';
import { Groups_MonitorsSumOrderByAggregateInput } from './groups-monitors-sum-order-by-aggregate.input';

@InputType()
export class Groups_MonitorsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    GroupId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => Groups_MonitorsCountOrderByAggregateInput, {nullable:true})
    _count?: Groups_MonitorsCountOrderByAggregateInput;

    @Field(() => Groups_MonitorsAvgOrderByAggregateInput, {nullable:true})
    _avg?: Groups_MonitorsAvgOrderByAggregateInput;

    @Field(() => Groups_MonitorsMaxOrderByAggregateInput, {nullable:true})
    _max?: Groups_MonitorsMaxOrderByAggregateInput;

    @Field(() => Groups_MonitorsMinOrderByAggregateInput, {nullable:true})
    _min?: Groups_MonitorsMinOrderByAggregateInput;

    @Field(() => Groups_MonitorsSumOrderByAggregateInput, {nullable:true})
    _sum?: Groups_MonitorsSumOrderByAggregateInput;
}
