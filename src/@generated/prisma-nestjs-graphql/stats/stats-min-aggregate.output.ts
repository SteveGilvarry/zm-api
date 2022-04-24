import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class StatsMinAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => Int, {nullable:true})
    ZoneId?: number;

    @Field(() => String, {nullable:true})
    EventId?: bigint | number;

    @Field(() => Int, {nullable:true})
    FrameId?: number;

    @Field(() => Int, {nullable:true})
    PixelDiff?: number;

    @Field(() => Int, {nullable:true})
    AlarmPixels?: number;

    @Field(() => Int, {nullable:true})
    FilterPixels?: number;

    @Field(() => Int, {nullable:true})
    BlobPixels?: number;

    @Field(() => Int, {nullable:true})
    Blobs?: number;

    @Field(() => Int, {nullable:true})
    MinBlobSize?: number;

    @Field(() => Int, {nullable:true})
    MaxBlobSize?: number;

    @Field(() => Int, {nullable:true})
    MinX?: number;

    @Field(() => Int, {nullable:true})
    MaxX?: number;

    @Field(() => Int, {nullable:true})
    MinY?: number;

    @Field(() => Int, {nullable:true})
    MaxY?: number;

    @Field(() => Int, {nullable:true})
    Score?: number;
}
