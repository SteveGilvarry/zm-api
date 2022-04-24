import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { ZonePresets_Type } from '../prisma/zone-presets-type.enum';
import { ZonePresets_Units } from '../prisma/zone-presets-units.enum';
import { ZonePresets_CheckMethod } from './zone-presets-check-method.enum';

@ObjectType()
export class ZonePresetsMaxAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => ZonePresets_Type, {nullable:true})
    Type?: keyof typeof ZonePresets_Type;

    @Field(() => ZonePresets_Units, {nullable:true})
    Units?: keyof typeof ZonePresets_Units;

    @Field(() => ZonePresets_CheckMethod, {nullable:true})
    CheckMethod?: keyof typeof ZonePresets_CheckMethod;

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
