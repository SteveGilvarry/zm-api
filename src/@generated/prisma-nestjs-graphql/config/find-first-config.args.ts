import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigWhereInput } from './config-where.input';
import { ConfigOrderByWithRelationInput } from './config-order-by-with-relation.input';
import { ConfigWhereUniqueInput } from './config-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ConfigScalarFieldEnum } from './config-scalar-field.enum';

@ArgsType()
export class FindFirstConfigArgs {

    @Field(() => ConfigWhereInput, {nullable:true})
    where?: ConfigWhereInput;

    @Field(() => [ConfigOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ConfigOrderByWithRelationInput>;

    @Field(() => ConfigWhereUniqueInput, {nullable:true})
    cursor?: ConfigWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [ConfigScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof ConfigScalarFieldEnum>;
}
