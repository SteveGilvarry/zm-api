import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Devices } from './users-devices.enum';

@InputType()
export class EnumUsers_DevicesFieldUpdateOperationsInput {

    @Field(() => Users_Devices, {nullable:true})
    set?: keyof typeof Users_Devices;
}
