import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereInput } from './models-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyModelsArgs {

    @Field(() => ModelsWhereInput, {nullable:true})
    @Type(() => ModelsWhereInput)
    where?: ModelsWhereInput;
}
