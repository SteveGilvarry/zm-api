import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Zones_Type } from '../prisma/zones-type.enum';
import { Zones_Units } from '../prisma/zones-units.enum';
import { Zones_CheckMethod } from './zones-check-method.enum';

@ObjectType()
export class Zones {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MonitorId!: number;

    @Field(() => String, {nullable:false,defaultValue:''})
    Name!: string;

    @Field(() => Zones_Type, {nullable:false,defaultValue:'Active'})
    Type!: keyof typeof Zones_Type;

    @Field(() => Zones_Units, {nullable:false,defaultValue:'Pixels'})
    Units!: keyof typeof Zones_Units;

    @Field(() => Int, {nullable:false,defaultValue:0})
    NumCoords!: number;

    @Field(() => String, {nullable:false})
    Coords!: string;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Area!: number;

    @Field(() => Int, {nullable:true,defaultValue:0})
    AlarmRGB!: number | null;

    @Field(() => Zones_CheckMethod, {nullable:false,defaultValue:'Blobs'})
    CheckMethod!: keyof typeof Zones_CheckMethod;

    @Field(() => Int, {nullable:true})
    MinPixelThreshold!: number | null;

    @Field(() => Int, {nullable:true})
    MaxPixelThreshold!: number | null;

    @Field(() => Int, {nullable:true})
    MinAlarmPixels!: number | null;

    @Field(() => Int, {nullable:true})
    MaxAlarmPixels!: number | null;

    @Field(() => Int, {nullable:true})
    FilterX!: number | null;

    @Field(() => Int, {nullable:true})
    FilterY!: number | null;

    @Field(() => Int, {nullable:true})
    MinFilterPixels!: number | null;

    @Field(() => Int, {nullable:true})
    MaxFilterPixels!: number | null;

    @Field(() => Int, {nullable:true})
    MinBlobPixels!: number | null;

    @Field(() => Int, {nullable:true})
    MaxBlobPixels!: number | null;

    @Field(() => Int, {nullable:true})
    MinBlobs!: number | null;

    @Field(() => Int, {nullable:true})
    MaxBlobs!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    OverloadFrames!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    ExtendAlarmFrames!: number;
}
