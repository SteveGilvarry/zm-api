import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Stats {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MonitorId!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    ZoneId!: number;

    @Field(() => String, {nullable:false})
    EventId!: bigint;

    @Field(() => Int, {nullable:false,defaultValue:0})
    FrameId!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    PixelDiff!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AlarmPixels!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    FilterPixels!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    BlobPixels!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Blobs!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MinBlobSize!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MaxBlobSize!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MinX!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MaxX!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MinY!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MaxY!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Score!: number;
}
