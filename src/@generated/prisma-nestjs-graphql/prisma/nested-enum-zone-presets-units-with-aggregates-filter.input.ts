import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_Units } from './zone-presets-units.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumZonePresets_UnitsFilter } from './nested-enum-zone-presets-units-filter.input';

@InputType()
export class NestedEnumZonePresets_UnitsWithAggregatesFilter {

    @Field(() => ZonePresets_Units, {nullable:true})
    equals?: keyof typeof ZonePresets_Units;

    @Field(() => [ZonePresets_Units], {nullable:true})
    in?: Array<keyof typeof ZonePresets_Units>;

    @Field(() => [ZonePresets_Units], {nullable:true})
    notIn?: Array<keyof typeof ZonePresets_Units>;

    @Field(() => NestedEnumZonePresets_UnitsWithAggregatesFilter, {nullable:true})
    not?: NestedEnumZonePresets_UnitsWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumZonePresets_UnitsFilter, {nullable:true})
    _min?: NestedEnumZonePresets_UnitsFilter;

    @Field(() => NestedEnumZonePresets_UnitsFilter, {nullable:true})
    _max?: NestedEnumZonePresets_UnitsFilter;
}
