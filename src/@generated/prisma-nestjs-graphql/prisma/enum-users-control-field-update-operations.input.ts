import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Control } from './users-control.enum';

@InputType()
export class EnumUsers_ControlFieldUpdateOperationsInput {

    @Field(() => Users_Control, {nullable:true})
    set?: keyof typeof Users_Control;
}
