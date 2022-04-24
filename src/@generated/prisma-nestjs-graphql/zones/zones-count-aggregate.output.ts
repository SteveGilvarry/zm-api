import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ZonesCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Type!: number;

    @Field(() => Int, {nullable:false})
    Units!: number;

    @Field(() => Int, {nullable:false})
    NumCoords!: number;

    @Field(() => Int, {nullable:false})
    Coords!: number;

    @Field(() => Int, {nullable:false})
    Area!: number;

    @Field(() => Int, {nullable:false})
    AlarmRGB!: number;

    @Field(() => Int, {nullable:false})
    CheckMethod!: number;

    @Field(() => Int, {nullable:false})
    MinPixelThreshold!: number;

    @Field(() => Int, {nullable:false})
    MaxPixelThreshold!: number;

    @Field(() => Int, {nullable:false})
    MinAlarmPixels!: number;

    @Field(() => Int, {nullable:false})
    MaxAlarmPixels!: number;

    @Field(() => Int, {nullable:false})
    FilterX!: number;

    @Field(() => Int, {nullable:false})
    FilterY!: number;

    @Field(() => Int, {nullable:false})
    MinFilterPixels!: number;

    @Field(() => Int, {nullable:false})
    MaxFilterPixels!: number;

    @Field(() => Int, {nullable:false})
    MinBlobPixels!: number;

    @Field(() => Int, {nullable:false})
    MaxBlobPixels!: number;

    @Field(() => Int, {nullable:false})
    MinBlobs!: number;

    @Field(() => Int, {nullable:false})
    MaxBlobs!: number;

    @Field(() => Int, {nullable:false})
    OverloadFrames!: number;

    @Field(() => Int, {nullable:false})
    ExtendAlarmFrames!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
