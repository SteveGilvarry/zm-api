import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class StatsAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    MonitorId?: number;

    @Field(() => Float, {nullable:true})
    ZoneId?: number;

    @Field(() => Float, {nullable:true})
    EventId?: number;

    @Field(() => Float, {nullable:true})
    FrameId?: number;

    @Field(() => Float, {nullable:true})
    PixelDiff?: number;

    @Field(() => Float, {nullable:true})
    AlarmPixels?: number;

    @Field(() => Float, {nullable:true})
    FilterPixels?: number;

    @Field(() => Float, {nullable:true})
    BlobPixels?: number;

    @Field(() => Float, {nullable:true})
    Blobs?: number;

    @Field(() => Float, {nullable:true})
    MinBlobSize?: number;

    @Field(() => Float, {nullable:true})
    MaxBlobSize?: number;

    @Field(() => Float, {nullable:true})
    MinX?: number;

    @Field(() => Float, {nullable:true})
    MaxX?: number;

    @Field(() => Float, {nullable:true})
    MinY?: number;

    @Field(() => Float, {nullable:true})
    MaxY?: number;

    @Field(() => Float, {nullable:true})
    Score?: number;
}
