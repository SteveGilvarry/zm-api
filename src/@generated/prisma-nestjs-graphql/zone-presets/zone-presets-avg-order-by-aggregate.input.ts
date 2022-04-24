import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class ZonePresetsAvgOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

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
}