import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Zones_Type } from '../prisma/zones-type.enum';
import { Zones_Units } from '../prisma/zones-units.enum';
import { Zones_CheckMethod } from './zones-check-method.enum';
import { ZonesCountAggregate } from './zones-count-aggregate.output';
import { ZonesAvgAggregate } from './zones-avg-aggregate.output';
import { ZonesSumAggregate } from './zones-sum-aggregate.output';
import { ZonesMinAggregate } from './zones-min-aggregate.output';
import { ZonesMaxAggregate } from './zones-max-aggregate.output';

@ObjectType()
export class ZonesGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Zones_Type, {nullable:false})
    Type!: keyof typeof Zones_Type;

    @Field(() => Zones_Units, {nullable:false})
    Units!: keyof typeof Zones_Units;

    @Field(() => Int, {nullable:false})
    NumCoords!: number;

    @Field(() => String, {nullable:false})
    Coords!: string;

    @Field(() => Int, {nullable:false})
    Area!: number;

    @Field(() => Int, {nullable:true})
    AlarmRGB?: number;

    @Field(() => Zones_CheckMethod, {nullable:false})
    CheckMethod!: keyof typeof Zones_CheckMethod;

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

    @Field(() => Int, {nullable:false})
    OverloadFrames!: number;

    @Field(() => Int, {nullable:false})
    ExtendAlarmFrames!: number;

    @Field(() => ZonesCountAggregate, {nullable:true})
    _count?: ZonesCountAggregate;

    @Field(() => ZonesAvgAggregate, {nullable:true})
    _avg?: ZonesAvgAggregate;

    @Field(() => ZonesSumAggregate, {nullable:true})
    _sum?: ZonesSumAggregate;

    @Field(() => ZonesMinAggregate, {nullable:true})
    _min?: ZonesMinAggregate;

    @Field(() => ZonesMaxAggregate, {nullable:true})
    _max?: ZonesMaxAggregate;
}
