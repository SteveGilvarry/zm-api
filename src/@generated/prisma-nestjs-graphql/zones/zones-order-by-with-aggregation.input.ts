import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { ZonesCountOrderByAggregateInput } from './zones-count-order-by-aggregate.input';
import { ZonesAvgOrderByAggregateInput } from './zones-avg-order-by-aggregate.input';
import { ZonesMaxOrderByAggregateInput } from './zones-max-order-by-aggregate.input';
import { ZonesMinOrderByAggregateInput } from './zones-min-order-by-aggregate.input';
import { ZonesSumOrderByAggregateInput } from './zones-sum-order-by-aggregate.input';

@InputType()
export class ZonesOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Units?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    NumCoords?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Coords?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Area?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmRGB?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CheckMethod?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinPixelThreshold?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxPixelThreshold?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinAlarmPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxAlarmPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FilterX?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FilterY?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinFilterPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxFilterPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinBlobPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxBlobPixels?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MinBlobs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxBlobs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    OverloadFrames?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ExtendAlarmFrames?: keyof typeof SortOrder;

    @Field(() => ZonesCountOrderByAggregateInput, {nullable:true})
    _count?: ZonesCountOrderByAggregateInput;

    @Field(() => ZonesAvgOrderByAggregateInput, {nullable:true})
    _avg?: ZonesAvgOrderByAggregateInput;

    @Field(() => ZonesMaxOrderByAggregateInput, {nullable:true})
    _max?: ZonesMaxOrderByAggregateInput;

    @Field(() => ZonesMinOrderByAggregateInput, {nullable:true})
    _min?: ZonesMinOrderByAggregateInput;

    @Field(() => ZonesSumOrderByAggregateInput, {nullable:true})
    _sum?: ZonesSumOrderByAggregateInput;
}
