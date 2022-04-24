import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsUpdateInput } from './models-update.input';
import { ModelsWhereUniqueInput } from './models-where-unique.input';

@ArgsType()
export class UpdateOneModelsArgs {

    @Field(() => ModelsUpdateInput, {nullable:false})
    data!: ModelsUpdateInput;

    @Field(() => ModelsWhereUniqueInput, {nullable:false})
    where!: ModelsWhereUniqueInput;
}
