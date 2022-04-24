import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { EnumZones_TypeWithAggregatesFilter } from '../prisma/enum-zones-type-with-aggregates-filter.input';
import { EnumZones_UnitsWithAggregatesFilter } from '../prisma/enum-zones-units-with-aggregates-filter.input';
import { IntNullableWithAggregatesFilter } from '../prisma/int-nullable-with-aggregates-filter.input';
import { EnumZones_CheckMethodWithAggregatesFilter } from '../prisma/enum-zones-check-method-with-aggregates-filter.input';

@InputType()
export class ZonesScalarWhereWithAggregatesInput {

    @Field(() => [ZonesScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<ZonesScalarWhereWithAggregatesInput>;

    @Field(() => [ZonesScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<ZonesScalarWhereWithAggregatesInput>;

    @Field(() => [ZonesScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<ZonesScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;

    @Field(() => EnumZones_TypeWithAggregatesFilter, {nullable:true})
    Type?: EnumZones_TypeWithAggregatesFilter;

    @Field(() => EnumZones_UnitsWithAggregatesFilter, {nullable:true})
    Units?: EnumZones_UnitsWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    NumCoords?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Coords?: StringWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Area?: IntWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    AlarmRGB?: IntNullableWithAggregatesFilter;

    @Field(() => EnumZones_CheckMethodWithAggregatesFilter, {nullable:true})
    CheckMethod?: EnumZones_CheckMethodWithAggregatesFilter;

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
