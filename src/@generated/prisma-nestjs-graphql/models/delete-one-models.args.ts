import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereUniqueInput } from './models-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneModelsArgs {

    @Field(() => ModelsWhereUniqueInput, {nullable:false})
    @Type(() => ModelsWhereUniqueInput)
    where!: ModelsWhereUniqueInput;
}
