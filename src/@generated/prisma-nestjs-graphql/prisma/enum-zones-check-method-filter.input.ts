import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_CheckMethod } from '../zones/zones-check-method.enum';
import { NestedEnumZones_CheckMethodFilter } from './nested-enum-zones-check-method-filter.input';

@InputType()
export class EnumZones_CheckMethodFilter {

    @Field(() => Zones_CheckMethod, {nullable:true})
    equals?: keyof typeof Zones_CheckMethod;

    @Field(() => [Zones_CheckMethod], {nullable:true})
    in?: Array<keyof typeof Zones_CheckMethod>;

    @Field(() => [Zones_CheckMethod], {nullable:true})
    notIn?: Array<keyof typeof Zones_CheckMethod>;

    @Field(() => NestedEnumZones_CheckMethodFilter, {nullable:true})
    not?: NestedEnumZones_CheckMethodFilter;
}
