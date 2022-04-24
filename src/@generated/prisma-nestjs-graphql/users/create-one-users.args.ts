import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { UsersCreateInput } from './users-create.input';

@ArgsType()
export class CreateOneUsersArgs {

    @Field(() => UsersCreateInput, {nullable:false})
    data!: UsersCreateInput;
}
