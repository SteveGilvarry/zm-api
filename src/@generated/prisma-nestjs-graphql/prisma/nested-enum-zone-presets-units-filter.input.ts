import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_Units } from './zone-presets-units.enum';

@InputType()
export class NestedEnumZonePresets_UnitsFilter {

    @Field(() => ZonePresets_Units, {nullable:true})
    equals?: keyof typeof ZonePresets_Units;

    @Field(() => [ZonePresets_Units], {nullable:true})
    in?: Array<keyof typeof ZonePresets_Units>;

    @Field(() => [ZonePresets_Units], {nullable:true})
    notIn?: Array<keyof typeof ZonePresets_Units>;

    @Field(() => NestedEnumZonePresets_UnitsFilter, {nullable:true})
    not?: NestedEnumZonePresets_UnitsFilter;
}
