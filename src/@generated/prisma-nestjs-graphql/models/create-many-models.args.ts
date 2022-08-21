import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsCreateManyInput } from './models-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyModelsArgs {

    @Field(() => [ModelsCreateManyInput], {nullable:false})
    @Type(() => ModelsCreateManyInput)
    data!: Array<ModelsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
