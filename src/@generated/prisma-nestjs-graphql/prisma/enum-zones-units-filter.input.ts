import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_Units } from './zones-units.enum';
import { NestedEnumZones_UnitsFilter } from './nested-enum-zones-units-filter.input';

@InputType()
export class EnumZones_UnitsFilter {

    @Field(() => Zones_Units, {nullable:true})
    equals?: keyof typeof Zones_Units;

    @Field(() => [Zones_Units], {nullable:true})
    in?: Array<keyof typeof Zones_Units>;

    @Field(() => [Zones_Units], {nullable:true})
    notIn?: Array<keyof typeof Zones_Units>;

    @Field(() => NestedEnumZones_UnitsFilter, {nullable:true})
    not?: NestedEnumZones_UnitsFilter;
}
