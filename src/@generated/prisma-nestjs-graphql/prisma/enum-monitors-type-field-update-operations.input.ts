import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Type } from './monitors-type.enum';

@InputType()
export class EnumMonitors_TypeFieldUpdateOperationsInput {

    @Field(() => Monitors_Type, {nullable:true})
    set?: keyof typeof Monitors_Type;
}
