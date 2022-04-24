import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_CheckMethod } from '../zone-presets/zone-presets-check-method.enum';

@InputType()
export class NestedEnumZonePresets_CheckMethodFilter {

    @Field(() => ZonePresets_CheckMethod, {nullable:true})
    equals?: keyof typeof ZonePresets_CheckMethod;

    @Field(() => [ZonePresets_CheckMethod], {nullable:true})
    in?: Array<keyof typeof ZonePresets_CheckMethod>;

    @Field(() => [ZonePresets_CheckMethod], {nullable:true})
    notIn?: Array<keyof typeof ZonePresets_CheckMethod>;

    @Field(() => NestedEnumZonePresets_CheckMethodFilter, {nullable:true})
    not?: NestedEnumZonePresets_CheckMethodFilter;
}
