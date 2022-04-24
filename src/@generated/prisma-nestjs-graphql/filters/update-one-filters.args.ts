import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersUpdateInput } from './filters-update.input';
import { FiltersWhereUniqueInput } from './filters-where-unique.input';

@ArgsType()
export class UpdateOneFiltersArgs {

    @Field(() => FiltersUpdateInput, {nullable:false})
    data!: FiltersUpdateInput;

    @Field(() => FiltersWhereUniqueInput, {nullable:false})
    where!: FiltersWhereUniqueInput;
}
