import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Devices } from './users-devices.enum';
import { NestedEnumUsers_DevicesFilter } from './nested-enum-users-devices-filter.input';

@InputType()
export class EnumUsers_DevicesFilter {

    @Field(() => Users_Devices, {nullable:true})
    equals?: keyof typeof Users_Devices;

    @Field(() => [Users_Devices], {nullable:true})
    in?: Array<keyof typeof Users_Devices>;

    @Field(() => [Users_Devices], {nullable:true})
    notIn?: Array<keyof typeof Users_Devices>;

    @Field(() => NestedEnumUsers_DevicesFilter, {nullable:true})
    not?: NestedEnumUsers_DevicesFilter;
}
