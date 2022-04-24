import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_Type } from './zone-presets-type.enum';

@InputType()
export class NestedEnumZonePresets_TypeFilter {

    @Field(() => ZonePresets_Type, {nullable:true})
    equals?: keyof typeof ZonePresets_Type;

    @Field(() => [ZonePresets_Type], {nullable:true})
    in?: Array<keyof typeof ZonePresets_Type>;

    @Field(() => [ZonePresets_Type], {nullable:true})
    notIn?: Array<keyof typeof ZonePresets_Type>;

    @Field(() => NestedEnumZonePresets_TypeFilter, {nullable:true})
    not?: NestedEnumZonePresets_TypeFilter;
}
