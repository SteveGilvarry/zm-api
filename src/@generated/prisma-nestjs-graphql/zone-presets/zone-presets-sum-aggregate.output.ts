import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ZonePresetsSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    MinPixelThreshold?: number;

    @Field(() => Int, {nullable:true})
    MaxPixelThreshold?: number;

    @Field(() => Int, {nullable:true})
    MinAlarmPixels?: number;

    @Field(() => Int, {nullable:true})
    MaxAlarmPixels?: number;

    @Field(() => Int, {nullable:true})
    FilterX?: number;

    @Field(() => Int, {nullable:true})
    FilterY?: number;

    @Field(() => Int, {nullable:true})
    MinFilterPixels?: number;

    @Field(() => Int, {nullable:true})
    MaxFilterPixels?: number;

    @Field(() => Int, {nullable:true})
    MinBlobPixels?: number;

    @Field(() => Int, {nullable:true})
    MaxBlobPixels?: number;

    @Field(() => Int, {nullable:true})
    MinBlobs?: number;

    @Field(() => Int, {nullable:true})
    MaxBlobs?: number;

    @Field(() => Int, {nullable:true})
    OverloadFrames?: number;

    @Field(() => Int, {nullable:true})
    ExtendAlarmFrames?: number;
}
