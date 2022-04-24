import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { EnumZonePresets_TypeWithAggregatesFilter } from '../prisma/enum-zone-presets-type-with-aggregates-filter.input';
import { EnumZonePresets_UnitsWithAggregatesFilter } from '../prisma/enum-zone-presets-units-with-aggregates-filter.input';
import { EnumZonePresets_CheckMethodWithAggregatesFilter } from '../prisma/enum-zone-presets-check-method-with-aggregates-filter.input';
import { IntNullableWithAggregatesFilter } from '../prisma/int-nullable-with-aggregates-filter.input';

@InputType()
export class ZonePresetsScalarWhereWithAggregatesInput {

    @Field(() => [ZonePresetsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<ZonePresetsScalarWhereWithAggregatesInput>;

    @Field(() => [ZonePresetsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<ZonePresetsScalarWhereWithAggregatesInput>;

    @Field(() => [ZonePresetsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<ZonePresetsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;

    @Field(() => EnumZonePresets_TypeWithAggregatesFilter, {nullable:true})
    Type?: EnumZonePresets_TypeWithAggregatesFilter;

    @Field(() => EnumZonePresets_UnitsWithAggregatesFilter, {nullable:true})
    Units?: EnumZonePresets_UnitsWithAggregatesFilter;

    @Field(() => EnumZonePresets_CheckMethodWithAggregatesFilter, {nullable:true})
    CheckMethod?: EnumZonePresets_CheckMethodWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MinPixelThreshold?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MaxPixelThreshold?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MinAlarmPixels?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MaxAlarmPixels?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    FilterX?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    FilterY?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MinFilterPixels?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MaxFilterPixels?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MinBlobPixels?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MaxBlobPixels?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MinBlobs?: IntNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    MaxBlobs?: IntNullableWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    OverloadFrames?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    ExtendAlarmFrames?: IntWithAggregatesFilter;
}
