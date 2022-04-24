import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { EnumZones_TypeFilter } from '../prisma/enum-zones-type-filter.input';
import { EnumZones_UnitsFilter } from '../prisma/enum-zones-units-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { EnumZones_CheckMethodFilter } from '../prisma/enum-zones-check-method-filter.input';

@InputType()
export class ZonesWhereInput {

    @Field(() => [ZonesWhereInput], {nullable:true})
    AND?: Array<ZonesWhereInput>;

    @Field(() => [ZonesWhereInput], {nullable:true})
    OR?: Array<ZonesWhereInput>;

    @Field(() => [ZonesWhereInput], {nullable:true})
    NOT?: Array<ZonesWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => EnumZones_TypeFilter, {nullable:true})
    Type?: EnumZones_TypeFilter;

    @Field(() => EnumZones_UnitsFilter, {nullable:true})
    Units?: EnumZones_UnitsFilter;

    @Field(() => IntFilter, {nullable:true})
    NumCoords?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Coords?: StringFilter;

    @Field(() => IntFilter, {nullable:true})
    Area?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    AlarmRGB?: IntNullableFilter;

    @Field(() => EnumZones_CheckMethodFilter, {nullable:true})
    CheckMethod?: EnumZones_CheckMethodFilter;

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
