import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { FramesCountOrderByAggregateInput } from './frames-count-order-by-aggregate.input';
import { FramesAvgOrderByAggregateInput } from './frames-avg-order-by-aggregate.input';
import { FramesMaxOrderByAggregateInput } from './frames-max-order-by-aggregate.input';
import { FramesMinOrderByAggregateInput } from './frames-min-order-by-aggregate.input';
import { FramesSumOrderByAggregateInput } from './frames-sum-order-by-aggregate.input';

@InputType()
export class FramesOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FrameId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TimeStamp?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Delta?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Score?: keyof typeof SortOrder;

    @Field(() => FramesCountOrderByAggregateInput, {nullable:true})
    _count?: FramesCountOrderByAggregateInput;

    @Field(() => FramesAvgOrderByAggregateInput, {nullable:true})
    _avg?: FramesAvgOrderByAggregateInput;

    @Field(() => FramesMaxOrderByAggregateInput, {nullable:true})
    _max?: FramesMaxOrderByAggregateInput;

    @Field(() => FramesMinOrderByAggregateInput, {nullable:true})
    _min?: FramesMinOrderByAggregateInput;

    @Field(() => FramesSumOrderByAggregateInput, {nullable:true})
    _sum?: FramesSumOrderByAggregateInput;
}
