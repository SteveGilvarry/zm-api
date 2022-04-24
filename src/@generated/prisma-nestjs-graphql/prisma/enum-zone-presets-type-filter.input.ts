import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_Type } from './zone-presets-type.enum';
import { NestedEnumZonePresets_TypeFilter } from './nested-enum-zone-presets-type-filter.input';

@InputType()
export class EnumZonePresets_TypeFilter {

    @Field(() => ZonePresets_Type, {nullable:true})
    equals?: keyof typeof ZonePresets_Type;

    @Field(() => [ZonePresets_Type], {nullable:true})
    in?: Array<keyof typeof ZonePresets_Type>;

    @Field(() => [ZonePresets_Type], {nullable:true})
    notIn?: Array<keyof typeof ZonePresets_Type>;

    @Field(() => NestedEnumZonePresets_TypeFilter, {nullable:true})
    not?: NestedEnumZonePresets_TypeFilter;
}
