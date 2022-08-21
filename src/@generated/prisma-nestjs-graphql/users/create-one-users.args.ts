import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { UsersCreateInput } from './users-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneUsersArgs {

    @Field(() => UsersCreateInput, {nullable:false})
    @Type(() => UsersCreateInput)
    data!: UsersCreateInput;
}
