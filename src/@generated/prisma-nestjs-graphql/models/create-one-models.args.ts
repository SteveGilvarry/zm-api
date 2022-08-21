import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsCreateInput } from './models-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneModelsArgs {

    @Field(() => ModelsCreateInput, {nullable:false})
    @Type(() => ModelsCreateInput)
    data!: ModelsCreateInput;
}
