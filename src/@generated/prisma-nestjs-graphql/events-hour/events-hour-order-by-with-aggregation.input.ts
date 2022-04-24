import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Events_HourCountOrderByAggregateInput } from './events-hour-count-order-by-aggregate.input';
import { Events_HourAvgOrderByAggregateInput } from './events-hour-avg-order-by-aggregate.input';
import { Events_HourMaxOrderByAggregateInput } from './events-hour-max-order-by-aggregate.input';
import { Events_HourMinOrderByAggregateInput } from './events-hour-min-order-by-aggregate.input';
import { Events_HourSumOrderByAggregateInput } from './events-hour-sum-order-by-aggregate.input';

@InputType()
export class Events_HourOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StartDateTime?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;

    @Field(() => Events_HourCountOrderByAggregateInput, {nullable:true})
    _count?: Events_HourCountOrderByAggregateInput;

    @Field(() => Events_HourAvgOrderByAggregateInput, {nullable:true})
    _avg?: Events_HourAvgOrderByAggregateInput;

    @Field(() => Events_HourMaxOrderByAggregateInput, {nullable:true})
    _max?: Events_HourMaxOrderByAggregateInput;

    @Field(() => Events_HourMinOrderByAggregateInput, {nullable:true})
    _min?: Events_HourMinOrderByAggregateInput;

    @Field(() => Events_HourSumOrderByAggregateInput, {nullable:true})
    _sum?: Events_HourSumOrderByAggregateInput;
}
