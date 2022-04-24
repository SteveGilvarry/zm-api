import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Function } from './monitors-function.enum';

@InputType()
export class EnumMonitors_FunctionFieldUpdateOperationsInput {

    @Field(() => Monitors_Function, {nullable:true})
    set?: keyof typeof Monitors_Function;
}
