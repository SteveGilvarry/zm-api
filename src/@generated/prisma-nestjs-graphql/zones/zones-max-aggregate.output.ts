import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Zones_Type } from '../prisma/zones-type.enum';
import { Zones_Units } from '../prisma/zones-units.enum';
import { Zones_CheckMethod } from './zones-check-method.enum';

@ObjectType()
export class ZonesMaxAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => Zones_Type, {nullable:true})
    Type?: keyof typeof Zones_Type;

    @Field(() => Zones_Units, {nullable:true})
    Units?: keyof typeof Zones_Units;

    @Field(() => Int, {nullable:true})
    NumCoords?: number;

    @Field(() => String, {nullable:true})
    Coords?: string;

    @Field(() => Int, {nullable:true})
    Area?: number;

    @Field(() => Int, {nullable:true})
    AlarmRGB?: number;

    @Field(() => Zones_CheckMethod, {nullable:true})
    CheckMethod?: keyof typeof Zones_CheckMethod;

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
