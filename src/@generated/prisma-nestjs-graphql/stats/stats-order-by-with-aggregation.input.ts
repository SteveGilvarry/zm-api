import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { StatsCountOrderByAggregateInput } from './stats-count-order-by-aggregate.input';
import { StatsAvgOrderByAggregateInput } from './stats-avg-order-by-aggregate.input';
import { StatsMaxOrderByAggregateInput } from './stats-max-order-by-aggregate.input';
import { StatsMinOrderByAggregateInput } from './stats-min-order-by-aggregate.input';
import { StatsSumOrderByAggregateInput } from './stats-sum-order-by-aggregate.input';

@InputType()
export class StatsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ZoneId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FrameId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    PixelDiff?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FilterPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    BlobPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Blobs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinBlobSize?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxBlobSize?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinX?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxX?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinY?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxY?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Score?: keyof typeof SortOrder;

    @Field(() => StatsCountOrderByAggregateInput, {nullable:true})
    _count?: StatsCountOrderByAggregateInput;

    @Field(() => StatsAvgOrderByAggregateInput, {nullable:true})
    _avg?: StatsAvgOrderByAggregateInput;

    @Field(() => StatsMaxOrderByAggregateInput, {nullable:true})
    _max?: StatsMaxOrderByAggregateInput;

    @Field(() => StatsMinOrderByAggregateInput, {nullable:true})
    _min?: StatsMinOrderByAggregateInput;

    @Field(() => StatsSumOrderByAggregateInput, {nullable:true})
    _sum?: StatsSumOrderByAggregateInput;
}
