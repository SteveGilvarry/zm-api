import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereInput } from './models-where.input';

@ArgsType()
export class DeleteManyModelsArgs {

    @Field(() => ModelsWhereInput, {nullable:true})
    where?: ModelsWhereInput;
}
