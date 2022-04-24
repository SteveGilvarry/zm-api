import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { StringFieldUpdateOperationsInput } from '../prisma/string-field-update-operations.input';
import { EnumDevices_TypeFieldUpdateOperationsInput } from '../prisma/enum-devices-type-field-update-operations.input';

@InputType()
export class DevicesUpdateInput {

    @Field(() => StringFieldUpdateOperationsInput, {nullable:true})
    Name?: StringFieldUpdateOperationsInput;

    @Field(() => EnumDevices_TypeFieldUpdateOperationsInput, {nullable:true})
    Type?: EnumDevices_TypeFieldUpdateOperationsInput;

    @Field(() => StringFieldUpdateOperationsInput, {nullable:true})
    KeyString?: StringFieldUpdateOperationsInput;
}
