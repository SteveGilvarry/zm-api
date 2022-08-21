import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsUpdateInput } from './models-update.input';
import { Type } from 'class-transformer';
import { ModelsWhereUniqueInput } from './models-where-unique.input';

@ArgsType()
export class UpdateOneModelsArgs {

    @Field(() => ModelsUpdateInput, {nullable:false})
    @Type(() => ModelsUpdateInput)
    data!: ModelsUpdateInput;

    @Field(() => ModelsWhereUniqueInput, {nullable:false})
    @Type(() => ModelsWhereUniqueInput)
    where!: ModelsWhereUniqueInput;
}
