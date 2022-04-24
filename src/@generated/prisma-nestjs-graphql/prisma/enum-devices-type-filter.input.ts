import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Devices_Type } from './devices-type.enum';
import { NestedEnumDevices_TypeFilter } from './nested-enum-devices-type-filter.input';

@InputType()
export class EnumDevices_TypeFilter {

    @Field(() => Devices_Type, {nullable:true})
    equals?: keyof typeof Devices_Type;

    @Field(() => [Devices_Type], {nullable:true})
    in?: Array<keyof typeof Devices_Type>;

    @Field(() => [Devices_Type], {nullable:true})
    notIn?: Array<keyof typeof Devices_Type>;

    @Field(() => NestedEnumDevices_TypeFilter, {nullable:true})
    not?: NestedEnumDevices_TypeFilter;
}
