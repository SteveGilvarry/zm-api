import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Type } from 'class-transformer';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { EnumMonitorPresets_TypeFilter } from '../prisma/enum-monitor-presets-type-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { DecimalNullableFilter } from '../prisma/decimal-nullable-filter.input';

@InputType()
export class MonitorPresetsWhereInput {

    @Field(() => [MonitorPresetsWhereInput], {nullable:true})
    @Type(() => MonitorPresetsWhereInput)
    AND?: Array<MonitorPresetsWhereInput>;

    @Field(() => [MonitorPresetsWhereInput], {nullable:true})
    @Type(() => MonitorPresetsWhereInput)
    OR?: Array<MonitorPresetsWhereInput>;

    @Field(() => [MonitorPresetsWhereInput], {nullable:true})
    @Type(() => MonitorPresetsWhereInput)
    NOT?: Array<MonitorPresetsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => EnumMonitorPresets_TypeFilter, {nullable:true})
    Type?: EnumMonitorPresets_TypeFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Device?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Channel?: StringNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    Format?: IntNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Protocol?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Method?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Host?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Port?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Path?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    SubPath?: StringNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    Width?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    Height?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    Palette?: IntNullableFilter;

    @Field(() => DecimalNullableFilter, {nullable:true})
    @Type(() => DecimalNullableFilter)
    MaxFPS?: DecimalNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    Controllable?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    ControlId?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    ControlDevice?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    ControlAddress?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    DefaultRate?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    DefaultScale?: IntFilter;
}
