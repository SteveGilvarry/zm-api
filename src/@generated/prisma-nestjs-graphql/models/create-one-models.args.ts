import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsCreateInput } from './models-create.input';

@ArgsType()
export class CreateOneModelsArgs {

    @Field(() => ModelsCreateInput, {nullable:false})
    data!: ModelsCreateInput;
}
