import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { UsersUpdateManyMutationInput } from './users-update-many-mutation.input';
import { UsersWhereInput } from './users-where.input';

@ArgsType()
export class UpdateManyUsersArgs {

    @Field(() => UsersUpdateManyMutationInput, {nullable:false})
    data!: UsersUpdateManyMutationInput;

    @Field(() => UsersWhereInput, {nullable:true})
    where?: UsersWhereInput;
}
