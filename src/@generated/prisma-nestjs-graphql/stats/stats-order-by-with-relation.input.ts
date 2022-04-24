import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class StatsOrderByWithRelationInput {

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
}
