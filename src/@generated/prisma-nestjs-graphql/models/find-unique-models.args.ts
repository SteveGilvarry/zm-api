import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereUniqueInput } from './models-where-unique.input';

@ArgsType()
export class FindUniqueModelsArgs {

    @Field(() => ModelsWhereUniqueInput, {nullable:false})
    where!: ModelsWhereUniqueInput;
}
