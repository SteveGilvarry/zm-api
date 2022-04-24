import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { UsersWhereUniqueInput } from './users-where-unique.input';
import { UsersCreateInput } from './users-create.input';
import { UsersUpdateInput } from './users-update.input';

@ArgsType()
export class UpsertOneUsersArgs {

    @Field(() => UsersWhereUniqueInput, {nullable:false})
    where!: UsersWhereUniqueInput;

    @Field(() => UsersCreateInput, {nullable:false})
    create!: UsersCreateInput;

    @Field(() => UsersUpdateInput, {nullable:false})
    update!: UsersUpdateInput;
}
