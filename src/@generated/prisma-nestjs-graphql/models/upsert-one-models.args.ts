import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereUniqueInput } from './models-where-unique.input';
import { Type } from 'class-transformer';
import { ModelsCreateInput } from './models-create.input';
import { ModelsUpdateInput } from './models-update.input';

@ArgsType()
export class UpsertOneModelsArgs {

    @Field(() => ModelsWhereUniqueInput, {nullable:false})
    @Type(() => ModelsWhereUniqueInput)
    where!: ModelsWhereUniqueInput;

    @Field(() => ModelsCreateInput, {nullable:false})
    @Type(() => ModelsCreateInput)
    create!: ModelsCreateInput;

    @Field(() => ModelsUpdateInput, {nullable:false})
    @Type(() => ModelsUpdateInput)
    update!: ModelsUpdateInput;
}
