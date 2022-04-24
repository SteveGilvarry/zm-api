import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Events_WeekCountOrderByAggregateInput } from './events-week-count-order-by-aggregate.input';
import { Events_WeekAvgOrderByAggregateInput } from './events-week-avg-order-by-aggregate.input';
import { Events_WeekMaxOrderByAggregateInput } from './events-week-max-order-by-aggregate.input';
import { Events_WeekMinOrderByAggregateInput } from './events-week-min-order-by-aggregate.input';
import { Events_WeekSumOrderByAggregateInput } from './events-week-sum-order-by-aggregate.input';

@InputType()
export class Events_WeekOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StartDateTime?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;

    @Field(() => Events_WeekCountOrderByAggregateInput, {nullable:true})
    _count?: Events_WeekCountOrderByAggregateInput;

    @Field(() => Events_WeekAvgOrderByAggregateInput, {nullable:true})
    _avg?: Events_WeekAvgOrderByAggregateInput;

    @Field(() => Events_WeekMaxOrderByAggregateInput, {nullable:true})
    _max?: Events_WeekMaxOrderByAggregateInput;

    @Field(() => Events_WeekMinOrderByAggregateInput, {nullable:true})
    _min?: Events_WeekMinOrderByAggregateInput;

    @Field(() => Events_WeekSumOrderByAggregateInput, {nullable:true})
    _sum?: Events_WeekSumOrderByAggregateInput;
}
