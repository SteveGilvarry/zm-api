import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Stream } from './users-stream.enum';

@InputType()
export class NestedEnumUsers_StreamFilter {

    @Field(() => Users_Stream, {nullable:true})
    equals?: keyof typeof Users_Stream;

    @Field(() => [Users_Stream], {nullable:true})
    in?: Array<keyof typeof Users_Stream>;

    @Field(() => [Users_Stream], {nullable:true})
    notIn?: Array<keyof typeof Users_Stream>;

    @Field(() => NestedEnumUsers_StreamFilter, {nullable:true})
    not?: NestedEnumUsers_StreamFilter;
}
