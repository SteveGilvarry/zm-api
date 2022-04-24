import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class ZonesAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    MonitorId?: number;

    @Field(() => Float, {nullable:true})
    NumCoords?: number;

    @Field(() => Float, {nullable:true})
    Area?: number;

    @Field(() => Float, {nullable:true})
    AlarmRGB?: number;

    @Field(() => Float, {nullable:true})
    MinPixelThreshold?: number;

    @Field(() => Float, {nullable:true})
    MaxPixelThreshold?: number;

    @Field(() => Float, {nullable:true})
    MinAlarmPixels?: number;

    @Field(() => Float, {nullable:true})
    MaxAlarmPixels?: number;

    @Field(() => Float, {nullable:true})
    FilterX?: number;

    @Field(() => Float, {nullable:true})
    FilterY?: number;

    @Field(() => Float, {nullable:true})
    MinFilterPixels?: number;

    @Field(() => Float, {nullable:true})
    MaxFilterPixels?: number;

    @Field(() => Float, {nullable:true})
    MinBlobPixels?: number;

    @Field(() => Float, {nullable:true})
    MaxBlobPixels?: number;

    @Field(() => Float, {nullable:true})
    MinBlobs?: number;

    @Field(() => Float, {nullable:true})
    MaxBlobs?: number;

    @Field(() => Float, {nullable:true})
    OverloadFrames?: number;

    @Field(() => Float, {nullable:true})
    ExtendAlarmFrames?: number;
}
