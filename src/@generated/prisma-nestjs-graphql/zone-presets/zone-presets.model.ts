import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { ZonePresets_Type } from '../prisma/zone-presets-type.enum';
import { ZonePresets_Units } from '../prisma/zone-presets-units.enum';
import { ZonePresets_CheckMethod } from './zone-presets-check-method.enum';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ZonePresets {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false,defaultValue:''})
    Name!: string;

    @Field(() => ZonePresets_Type, {nullable:false,defaultValue:'Active'})
    Type!: keyof typeof ZonePresets_Type;

    @Field(() => ZonePresets_Units, {nullable:false,defaultValue:'Pixels'})
    Units!: keyof typeof ZonePresets_Units;

    @Field(() => ZonePresets_CheckMethod, {nullable:false,defaultValue:'Blobs'})
    CheckMethod!: keyof typeof ZonePresets_CheckMethod;

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
