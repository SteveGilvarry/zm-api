import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_System } from './users-system.enum';

@InputType()
export class EnumUsers_SystemFieldUpdateOperationsInput {

    @Field(() => Users_System, {nullable:true})
    set?: keyof typeof Users_System;
}
