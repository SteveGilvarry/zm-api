import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Stream } from './users-stream.enum';

@InputType()
export class EnumUsers_StreamFieldUpdateOperationsInput {

    @Field(() => Users_Stream, {nullable:true})
    set?: keyof typeof Users_Stream;
}
