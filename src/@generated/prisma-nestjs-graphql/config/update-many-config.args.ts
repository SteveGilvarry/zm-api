import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigUpdateManyMutationInput } from './config-update-many-mutation.input';
import { ConfigWhereInput } from './config-where.input';

@ArgsType()
export class UpdateManyConfigArgs {

    @Field(() => ConfigUpdateManyMutationInput, {nullable:false})
    data!: ConfigUpdateManyMutationInput;

    @Field(() => ConfigWhereInput, {nullable:true})
    where?: ConfigWhereInput;
}
