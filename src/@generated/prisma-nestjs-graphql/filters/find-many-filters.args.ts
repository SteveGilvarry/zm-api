import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereInput } from './filters-where.input';
import { FiltersOrderByWithRelationInput } from './filters-order-by-with-relation.input';
import { FiltersWhereUniqueInput } from './filters-where-unique.input';
import { Int } from '@nestjs/graphql';
import { FiltersScalarFieldEnum } from './filters-scalar-field.enum';

@ArgsType()
export class FindManyFiltersArgs {

    @Field(() => FiltersWhereInput, {nullable:true})
    where?: FiltersWhereInput;

    @Field(() => [FiltersOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<FiltersOrderByWithRelationInput>;

    @Field(() => FiltersWhereUniqueInput, {nullable:true})
    cursor?: FiltersWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [FiltersScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof FiltersScalarFieldEnum>;
}
