import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Controls_Type } from './controls-type.enum';

@InputType()
export class EnumControls_TypeFieldUpdateOperationsInput {

    @Field(() => Controls_Type, {nullable:true})
    set?: keyof typeof Controls_Type;
}
