import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsUpdateManyMutationInput } from './models-update-many-mutation.input';
import { Type } from 'class-transformer';
import { ModelsWhereInput } from './models-where.input';

@ArgsType()
export class UpdateManyModelsArgs {

    @Field(() => ModelsUpdateManyMutationInput, {nullable:false})
    @Type(() => ModelsUpdateManyMutationInput)
    data!: ModelsUpdateManyMutationInput;

    @Field(() => ModelsWhereInput, {nullable:true})
    @Type(() => ModelsWhereInput)
    where?: ModelsWhereInput;
}
