import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { ZonePresetsCountOrderByAggregateInput } from './zone-presets-count-order-by-aggregate.input';
import { ZonePresetsAvgOrderByAggregateInput } from './zone-presets-avg-order-by-aggregate.input';
import { ZonePresetsMaxOrderByAggregateInput } from './zone-presets-max-order-by-aggregate.input';
import { ZonePresetsMinOrderByAggregateInput } from './zone-presets-min-order-by-aggregate.input';
import { ZonePresetsSumOrderByAggregateInput } from './zone-presets-sum-order-by-aggregate.input';

@InputType()
export class ZonePresetsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Units?: keyof typeof SortOrder;

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

    @Field(() => ZonePresetsCountOrderByAggregateInput, {nullable:true})
    _count?: ZonePresetsCountOrderByAggregateInput;

    @Field(() => ZonePresetsAvgOrderByAggregateInput, {nullable:true})
    _avg?: ZonePresetsAvgOrderByAggregateInput;

    @Field(() => ZonePresetsMaxOrderByAggregateInput, {nullable:true})
    _max?: ZonePresetsMaxOrderByAggregateInput;

    @Field(() => ZonePresetsMinOrderByAggregateInput, {nullable:true})
    _min?: ZonePresetsMinOrderByAggregateInput;

    @Field(() => ZonePresetsSumOrderByAggregateInput, {nullable:true})
    _sum?: ZonePresetsSumOrderByAggregateInput;
}
