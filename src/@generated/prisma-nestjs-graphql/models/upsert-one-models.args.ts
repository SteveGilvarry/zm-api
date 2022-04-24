import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereUniqueInput } from './models-where-unique.input';
import { ModelsCreateInput } from './models-create.input';
import { ModelsUpdateInput } from './models-update.input';

@ArgsType()
export class UpsertOneModelsArgs {

    @Field(() => ModelsWhereUniqueInput, {nullable:false})
    where!: ModelsWhereUniqueInput;

    @Field(() => ModelsCreateInput, {nullable:false})
    create!: ModelsCreateInput;

    @Field(() => ModelsUpdateInput, {nullable:false})
    update!: ModelsUpdateInput;
}
