import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_CheckMethod } from '../zone-presets/zone-presets-check-method.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumZonePresets_CheckMethodFilter } from './nested-enum-zone-presets-check-method-filter.input';

@InputType()
export class NestedEnumZonePresets_CheckMethodWithAggregatesFilter {

    @Field(() => ZonePresets_CheckMethod, {nullable:true})
    equals?: keyof typeof ZonePresets_CheckMethod;

    @Field(() => [ZonePresets_CheckMethod], {nullable:true})
    in?: Array<keyof typeof ZonePresets_CheckMethod>;

    @Field(() => [ZonePresets_CheckMethod], {nullable:true})
    notIn?: Array<keyof typeof ZonePresets_CheckMethod>;

    @Field(() => NestedEnumZonePresets_CheckMethodWithAggregatesFilter, {nullable:true})
    not?: NestedEnumZonePresets_CheckMethodWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumZonePresets_CheckMethodFilter, {nullable:true})
    _min?: NestedEnumZonePresets_CheckMethodFilter;

    @Field(() => NestedEnumZonePresets_CheckMethodFilter, {nullable:true})
    _max?: NestedEnumZonePresets_CheckMethodFilter;
}
