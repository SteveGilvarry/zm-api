import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SessionsUpdateManyMutationInput } from './sessions-update-many-mutation.input';
import { SessionsWhereInput } from './sessions-where.input';

@ArgsType()
export class UpdateManySessionsArgs {

    @Field(() => SessionsUpdateManyMutationInput, {nullable:false})
    data!: SessionsUpdateManyMutationInput;

    @Field(() => SessionsWhereInput, {nullable:true})
    where?: SessionsWhereInput;
}
