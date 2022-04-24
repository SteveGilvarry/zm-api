import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class StatsCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    ZoneId!: number;

    @Field(() => Int, {nullable:false})
    EventId!: number;

    @Field(() => Int, {nullable:false})
    FrameId!: number;

    @Field(() => Int, {nullable:false})
    PixelDiff!: number;

    @Field(() => Int, {nullable:false})
    AlarmPixels!: number;

    @Field(() => Int, {nullable:false})
    FilterPixels!: number;

    @Field(() => Int, {nullable:false})
    BlobPixels!: number;

    @Field(() => Int, {nullable:false})
    Blobs!: number;

    @Field(() => Int, {nullable:false})
    MinBlobSize!: number;

    @Field(() => Int, {nullable:false})
    MaxBlobSize!: number;

    @Field(() => Int, {nullable:false})
    MinX!: number;

    @Field(() => Int, {nullable:false})
    MaxX!: number;

    @Field(() => Int, {nullable:false})
    MinY!: number;

    @Field(() => Int, {nullable:false})
    MaxY!: number;

    @Field(() => Int, {nullable:false})
    Score!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
