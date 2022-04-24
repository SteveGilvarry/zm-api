import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsUpdateManyMutationInput } from './models-update-many-mutation.input';
import { ModelsWhereInput } from './models-where.input';

@ArgsType()
export class UpdateManyModelsArgs {

    @Field(() => ModelsUpdateManyMutationInput, {nullable:false})
    data!: ModelsUpdateManyMutationInput;

    @Field(() => ModelsWhereInput, {nullable:true})
    where?: ModelsWhereInput;
}
