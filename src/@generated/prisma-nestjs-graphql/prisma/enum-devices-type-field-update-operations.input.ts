import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Devices_Type } from './devices-type.enum';

@InputType()
export class EnumDevices_TypeFieldUpdateOperationsInput {

    @Field(() => Devices_Type, {nullable:true})
    set?: keyof typeof Devices_Type;
}
