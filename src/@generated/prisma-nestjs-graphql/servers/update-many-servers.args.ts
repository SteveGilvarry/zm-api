import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersUpdateManyMutationInput } from './servers-update-many-mutation.input';
import { ServersWhereInput } from './servers-where.input';

@ArgsType()
export class UpdateManyServersArgs {

    @Field(() => ServersUpdateManyMutationInput, {nullable:false})
    data!: ServersUpdateManyMutationInput;

    @Field(() => ServersWhereInput, {nullable:true})
    where?: ServersWhereInput;
}
