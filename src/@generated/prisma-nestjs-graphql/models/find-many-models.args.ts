import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereInput } from './models-where.input';
import { ModelsOrderByWithRelationInput } from './models-order-by-with-relation.input';
import { ModelsWhereUniqueInput } from './models-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ModelsScalarFieldEnum } from './models-scalar-field.enum';

@ArgsType()
export class FindManyModelsArgs {

    @Field(() => ModelsWhereInput, {nullable:true})
    where?: ModelsWhereInput;

    @Field(() => [ModelsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ModelsOrderByWithRelationInput>;

    @Field(() => ModelsWhereUniqueInput, {nullable:true})
    cursor?: ModelsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [ModelsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof ModelsScalarFieldEnum>;
}
