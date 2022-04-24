import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { ZonePresets_Type } from '../prisma/zone-presets-type.enum';
import { ZonePresets_Units } from '../prisma/zone-presets-units.enum';
import { ZonePresets_CheckMethod } from './zone-presets-check-method.enum';
import { ZonePresetsCountAggregate } from './zone-presets-count-aggregate.output';
import { ZonePresetsAvgAggregate } from './zone-presets-avg-aggregate.output';
import { ZonePresetsSumAggregate } from './zone-presets-sum-aggregate.output';
import { ZonePresetsMinAggregate } from './zone-presets-min-aggregate.output';
import { ZonePresetsMaxAggregate } from './zone-presets-max-aggregate.output';

@ObjectType()
export class ZonePresetsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => ZonePresets_Type, {nullable:false})
    Type!: keyof typeof ZonePresets_Type;

    @Field(() => ZonePresets_Units, {nullable:false})
    Units!: keyof typeof ZonePresets_Units;

    @Field(() => ZonePresets_CheckMethod, {nullable:false})
    CheckMethod!: keyof typeof ZonePresets_CheckMethod;

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

    @Field(() => ZonePresetsCountAggregate, {nullable:true})
    _count?: ZonePresetsCountAggregate;

    @Field(() => ZonePresetsAvgAggregate, {nullable:true})
    _avg?: ZonePresetsAvgAggregate;

    @Field(() => ZonePresetsSumAggregate, {nullable:true})
    _sum?: ZonePresetsSumAggregate;

    @Field(() => ZonePresetsMinAggregate, {nullable:true})
    _min?: ZonePresetsMinAggregate;

    @Field(() => ZonePresetsMaxAggregate, {nullable:true})
    _max?: ZonePresetsMaxAggregate;
}
