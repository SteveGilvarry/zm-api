import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_Type } from './zones-type.enum';
import { NestedEnumZones_TypeFilter } from './nested-enum-zones-type-filter.input';

@InputType()
export class EnumZones_TypeFilter {

    @Field(() => Zones_Type, {nullable:true})
    equals?: keyof typeof Zones_Type;

    @Field(() => [Zones_Type], {nullable:true})
    in?: Array<keyof typeof Zones_Type>;

    @Field(() => [Zones_Type], {nullable:true})
    notIn?: Array<keyof typeof Zones_Type>;

    @Field(() => NestedEnumZones_TypeFilter, {nullable:true})
    not?: NestedEnumZones_TypeFilter;
}
