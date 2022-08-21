import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { Monitor_StatusCountOrderByAggregateInput } from './monitor-status-count-order-by-aggregate.input';
import { Type } from 'class-transformer';
import { Monitor_StatusAvgOrderByAggregateInput } from './monitor-status-avg-order-by-aggregate.input';
import { Monitor_StatusMaxOrderByAggregateInput } from './monitor-status-max-order-by-aggregate.input';
import { Monitor_StatusMinOrderByAggregateInput } from './monitor-status-min-order-by-aggregate.input';
import { Monitor_StatusSumOrderByAggregateInput } from './monitor-status-sum-order-by-aggregate.input';

@InputType()
export class Monitor_StatusOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Status?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CaptureFPS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AnalysisFPS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CaptureBandwidth?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DayEventDiskSpace?: keyof typeof SortOrder;

    @Field(() => Monitor_StatusCountOrderByAggregateInput, {nullable:true})
    @Type(() => Monitor_StatusCountOrderByAggregateInput)
    _count?: Monitor_StatusCountOrderByAggregateInput;

    @Field(() => Monitor_StatusAvgOrderByAggregateInput, {nullable:true})
    @Type(() => Monitor_StatusAvgOrderByAggregateInput)
    _avg?: Monitor_StatusAvgOrderByAggregateInput;

    @Field(() => Monitor_StatusMaxOrderByAggregateInput, {nullable:true})
    @Type(() => Monitor_StatusMaxOrderByAggregateInput)
    _max?: Monitor_StatusMaxOrderByAggregateInput;

    @Field(() => Monitor_StatusMinOrderByAggregateInput, {nullable:true})
    @Type(() => Monitor_StatusMinOrderByAggregateInput)
    _min?: Monitor_StatusMinOrderByAggregateInput;

    @Field(() => Monitor_StatusSumOrderByAggregateInput, {nullable:true})
    @Type(() => Monitor_StatusSumOrderByAggregateInput)
    _sum?: Monitor_StatusSumOrderByAggregateInput;
}
