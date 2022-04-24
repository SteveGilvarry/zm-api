import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Devices_Type } from './devices-type.enum';
import { NestedEnumDevices_TypeWithAggregatesFilter } from './nested-enum-devices-type-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumDevices_TypeFilter } from './nested-enum-devices-type-filter.input';

@InputType()
export class EnumDevices_TypeWithAggregatesFilter {

    @Field(() => Devices_Type, {nullable:true})
    equals?: keyof typeof Devices_Type;

    @Field(() => [Devices_Type], {nullable:true})
    in?: Array<keyof typeof Devices_Type>;

    @Field(() => [Devices_Type], {nullable:true})
    notIn?: Array<keyof typeof Devices_Type>;

    @Field(() => NestedEnumDevices_TypeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumDevices_TypeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumDevices_TypeFilter, {nullable:true})
    _min?: NestedEnumDevices_TypeFilter;

    @Field(() => NestedEnumDevices_TypeFilter, {nullable:true})
    _max?: NestedEnumDevices_TypeFilter;
}
