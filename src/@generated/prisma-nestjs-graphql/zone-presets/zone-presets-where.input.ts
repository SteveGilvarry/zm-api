import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { EnumZonePresets_TypeFilter } from '../prisma/enum-zone-presets-type-filter.input';
import { EnumZonePresets_UnitsFilter } from '../prisma/enum-zone-presets-units-filter.input';
import { EnumZonePresets_CheckMethodFilter } from '../prisma/enum-zone-presets-check-method-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';

@InputType()
export class ZonePresetsWhereInput {

    @Field(() => [ZonePresetsWhereInput], {nullable:true})
    AND?: Array<ZonePresetsWhereInput>;

    @Field(() => [ZonePresetsWhereInput], {nullable:true})
    OR?: Array<ZonePresetsWhereInput>;

    @Field(() => [ZonePresetsWhereInput], {nullable:true})
    NOT?: Array<ZonePresetsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => EnumZonePresets_TypeFilter, {nullable:true})
    Type?: EnumZonePresets_TypeFilter;

    @Field(() => EnumZonePresets_UnitsFilter, {nullable:true})
    Units?: EnumZonePresets_UnitsFilter;

    @Field(() => EnumZonePresets_CheckMethodFilter, {nullable:true})
    CheckMethod?: EnumZonePresets_CheckMethodFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinPixelThreshold?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxPixelThreshold?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinAlarmPixels?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxAlarmPixels?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    FilterX?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    FilterY?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinFilterPixels?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxFilterPixels?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinBlobPixels?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxBlobPixels?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MinBlobs?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxBlobs?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    OverloadFrames?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    ExtendAlarmFrames?: IntFilter;
}
