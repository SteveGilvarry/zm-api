import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Storage_Type } from './storage-type.enum';

@InputType()
export class EnumStorage_TypeFieldUpdateOperationsInput {

    @Field(() => Storage_Type, {nullable:true})
    set?: keyof typeof Storage_Type;
}
