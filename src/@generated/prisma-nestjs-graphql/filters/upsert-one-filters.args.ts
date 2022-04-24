import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereUniqueInput } from './filters-where-unique.input';
import { FiltersCreateInput } from './filters-create.input';
import { FiltersUpdateInput } from './filters-update.input';

@ArgsType()
export class UpsertOneFiltersArgs {

    @Field(() => FiltersWhereUniqueInput, {nullable:false})
    where!: FiltersWhereUniqueInput;

    @Field(() => FiltersCreateInput, {nullable:false})
    create!: FiltersCreateInput;

    @Field(() => FiltersUpdateInput, {nullable:false})
    update!: FiltersUpdateInput;
}
