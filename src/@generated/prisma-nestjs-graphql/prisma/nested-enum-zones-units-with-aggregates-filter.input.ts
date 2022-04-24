import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_Units } from './zones-units.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumZones_UnitsFilter } from './nested-enum-zones-units-filter.input';

@InputType()
export class NestedEnumZones_UnitsWithAggregatesFilter {

    @Field(() => Zones_Units, {nullable:true})
    equals?: keyof typeof Zones_Units;

    @Field(() => [Zones_Units], {nullable:true})
    in?: Array<keyof typeof Zones_Units>;

    @Field(() => [Zones_Units], {nullable:true})
    notIn?: Array<keyof typeof Zones_Units>;

    @Field(() => NestedEnumZones_UnitsWithAggregatesFilter, {nullable:true})
    not?: NestedEnumZones_UnitsWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumZones_UnitsFilter, {nullable:true})
    _min?: NestedEnumZones_UnitsFilter;

    @Field(() => NestedEnumZones_UnitsFilter, {nullable:true})
    _max?: NestedEnumZones_UnitsFilter;
}
