import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Storage_Scheme } from './storage-scheme.enum';

@InputType()
export class EnumStorage_SchemeFieldUpdateOperationsInput {

    @Field(() => Storage_Scheme, {nullable:true})
    set?: keyof typeof Storage_Scheme;
}
