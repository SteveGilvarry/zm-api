import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Devices } from './users-devices.enum';

@InputType()
export class NestedEnumUsers_DevicesFilter {

    @Field(() => Users_Devices, {nullable:true})
    equals?: keyof typeof Users_Devices;

    @Field(() => [Users_Devices], {nullable:true})
    in?: Array<keyof typeof Users_Devices>;

    @Field(() => [Users_Devices], {nullable:true})
    notIn?: Array<keyof typeof Users_Devices>;

    @Field(() => NestedEnumUsers_DevicesFilter, {nullable:true})
    not?: NestedEnumUsers_DevicesFilter;
}
