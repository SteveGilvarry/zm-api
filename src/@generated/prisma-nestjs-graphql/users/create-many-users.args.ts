import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { UsersCreateManyInput } from './users-create-many.input';

@ArgsType()
export class CreateManyUsersArgs {

    @Field(() => [UsersCreateManyInput], {nullable:false})
    data!: Array<UsersCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
