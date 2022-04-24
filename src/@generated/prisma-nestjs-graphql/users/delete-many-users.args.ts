import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { UsersWhereInput } from './users-where.input';

@ArgsType()
export class DeleteManyUsersArgs {

    @Field(() => UsersWhereInput, {nullable:true})
    where?: UsersWhereInput;
}
