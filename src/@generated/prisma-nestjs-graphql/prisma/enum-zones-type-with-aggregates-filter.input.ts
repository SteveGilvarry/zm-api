import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_Type } from './zones-type.enum';
import { NestedEnumZones_TypeWithAggregatesFilter } from './nested-enum-zones-type-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumZones_TypeFilter } from './nested-enum-zones-type-filter.input';

@InputType()
export class EnumZones_TypeWithAggregatesFilter {

    @Field(() => Zones_Type, {nullable:true})
    equals?: keyof typeof Zones_Type;

    @Field(() => [Zones_Type], {nullable:true})
    in?: Array<keyof typeof Zones_Type>;

    @Field(() => [Zones_Type], {nullable:true})
    notIn?: Array<keyof typeof Zones_Type>;

    @Field(() => NestedEnumZones_TypeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumZones_TypeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumZones_TypeFilter, {nullable:true})
    _min?: NestedEnumZones_TypeFilter;

    @Field(() => NestedEnumZones_TypeFilter, {nullable:true})
    _max?: NestedEnumZones_TypeFilter;
}
