import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_CheckMethod } from '../zones/zones-check-method.enum';
import { NestedEnumZones_CheckMethodWithAggregatesFilter } from './nested-enum-zones-check-method-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumZones_CheckMethodFilter } from './nested-enum-zones-check-method-filter.input';

@InputType()
export class EnumZones_CheckMethodWithAggregatesFilter {

    @Field(() => Zones_CheckMethod, {nullable:true})
    equals?: keyof typeof Zones_CheckMethod;

    @Field(() => [Zones_CheckMethod], {nullable:true})
    in?: Array<keyof typeof Zones_CheckMethod>;

    @Field(() => [Zones_CheckMethod], {nullable:true})
    notIn?: Array<keyof typeof Zones_CheckMethod>;

    @Field(() => NestedEnumZones_CheckMethodWithAggregatesFilter, {nullable:true})
    not?: NestedEnumZones_CheckMethodWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumZones_CheckMethodFilter, {nullable:true})
    _min?: NestedEnumZones_CheckMethodFilter;

    @Field(() => NestedEnumZones_CheckMethodFilter, {nullable:true})
    _max?: NestedEnumZones_CheckMethodFilter;
}
