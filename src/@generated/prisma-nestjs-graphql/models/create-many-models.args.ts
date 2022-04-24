import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsCreateManyInput } from './models-create-many.input';

@ArgsType()
export class CreateManyModelsArgs {

    @Field(() => [ModelsCreateManyInput], {nullable:false})
    data!: Array<ModelsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
