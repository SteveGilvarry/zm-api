import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { UsersUpdateInput } from './users-update.input';
import { UsersWhereUniqueInput } from './users-where-unique.input';

@ArgsType()
export class UpdateOneUsersArgs {

    @Field(() => UsersUpdateInput, {nullable:false})
    data!: UsersUpdateInput;

    @Field(() => UsersWhereUniqueInput, {nullable:false})
    where!: UsersWhereUniqueInput;
}
