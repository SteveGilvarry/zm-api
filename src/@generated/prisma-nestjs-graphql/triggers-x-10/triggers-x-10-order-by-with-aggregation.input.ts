import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { TriggersX10CountOrderByAggregateInput } from './triggers-x-10-count-order-by-aggregate.input';
import { TriggersX10AvgOrderByAggregateInput } from './triggers-x-10-avg-order-by-aggregate.input';
import { TriggersX10MaxOrderByAggregateInput } from './triggers-x-10-max-order-by-aggregate.input';
import { TriggersX10MinOrderByAggregateInput } from './triggers-x-10-min-order-by-aggregate.input';
import { TriggersX10SumOrderByAggregateInput } from './triggers-x-10-sum-order-by-aggregate.input';

@InputType()
export class TriggersX10OrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Activation?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmInput?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmOutput?: keyof typeof SortOrder;

    @Field(() => TriggersX10CountOrderByAggregateInput, {nullable:true})
    _count?: TriggersX10CountOrderByAggregateInput;

    @Field(() => TriggersX10AvgOrderByAggregateInput, {nullable:true})
    _avg?: TriggersX10AvgOrderByAggregateInput;

    @Field(() => TriggersX10MaxOrderByAggregateInput, {nullable:true})
    _max?: TriggersX10MaxOrderByAggregateInput;

    @Field(() => TriggersX10MinOrderByAggregateInput, {nullable:true})
    _min?: TriggersX10MinOrderByAggregateInput;

    @Field(() => TriggersX10SumOrderByAggregateInput, {nullable:true})
    _sum?: TriggersX10SumOrderByAggregateInput;
}
