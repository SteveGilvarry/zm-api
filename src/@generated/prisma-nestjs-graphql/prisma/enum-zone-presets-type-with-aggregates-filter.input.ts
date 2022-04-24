import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_Type } from './zone-presets-type.enum';
import { NestedEnumZonePresets_TypeWithAggregatesFilter } from './nested-enum-zone-presets-type-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumZonePresets_TypeFilter } from './nested-enum-zone-presets-type-filter.input';

@InputType()
export class EnumZonePresets_TypeWithAggregatesFilter {

    @Field(() => ZonePresets_Type, {nullable:true})
    equals?: keyof typeof ZonePresets_Type;

    @Field(() => [ZonePresets_Type], {nullable:true})
    in?: Array<keyof typeof ZonePresets_Type>;

    @Field(() => [ZonePresets_Type], {nullable:true})
    notIn?: Array<keyof typeof ZonePresets_Type>;

    @Field(() => NestedEnumZonePresets_TypeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumZonePresets_TypeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumZonePresets_TypeFilter, {nullable:true})
    _min?: NestedEnumZonePresets_TypeFilter;

    @Field(() => NestedEnumZonePresets_TypeFilter, {nullable:true})
    _max?: NestedEnumZonePresets_TypeFilter;
}
