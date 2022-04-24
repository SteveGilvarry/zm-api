import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { UsersWhereUniqueInput } from './users-where-unique.input';

@ArgsType()
export class FindUniqueUsersArgs {

    @Field(() => UsersWhereUniqueInput, {nullable:false})
    where!: UsersWhereUniqueInput;
}
