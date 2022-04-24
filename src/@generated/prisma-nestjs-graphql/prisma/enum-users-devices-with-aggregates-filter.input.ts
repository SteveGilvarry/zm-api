import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Devices } from './users-devices.enum';
import { NestedEnumUsers_DevicesWithAggregatesFilter } from './nested-enum-users-devices-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumUsers_DevicesFilter } from './nested-enum-users-devices-filter.input';

@InputType()
export class EnumUsers_DevicesWithAggregatesFilter {

    @Field(() => Users_Devices, {nullable:true})
    equals?: keyof typeof Users_Devices;

    @Field(() => [Users_Devices], {nullable:true})
    in?: Array<keyof typeof Users_Devices>;

    @Field(() => [Users_Devices], {nullable:true})
    notIn?: Array<keyof typeof Users_Devices>;

    @Field(() => NestedEnumUsers_DevicesWithAggregatesFilter, {nullable:true})
    not?: NestedEnumUsers_DevicesWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumUsers_DevicesFilter, {nullable:true})
    _min?: NestedEnumUsers_DevicesFilter;

    @Field(() => NestedEnumUsers_DevicesFilter, {nullable:true})
    _max?: NestedEnumUsers_DevicesFilter;
}
