import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_CheckMethod } from '../zone-presets/zone-presets-check-method.enum';
import { NestedEnumZonePresets_CheckMethodFilter } from './nested-enum-zone-presets-check-method-filter.input';

@InputType()
export class EnumZonePresets_CheckMethodFilter {

    @Field(() => ZonePresets_CheckMethod, {nullable:true})
    equals?: keyof typeof ZonePresets_CheckMethod;

    @Field(() => [ZonePresets_CheckMethod], {nullable:true})
    in?: Array<keyof typeof ZonePresets_CheckMethod>;

    @Field(() => [ZonePresets_CheckMethod], {nullable:true})
    notIn?: Array<keyof typeof ZonePresets_CheckMethod>;

    @Field(() => NestedEnumZonePresets_CheckMethodFilter, {nullable:true})
    not?: NestedEnumZonePresets_CheckMethodFilter;
}
