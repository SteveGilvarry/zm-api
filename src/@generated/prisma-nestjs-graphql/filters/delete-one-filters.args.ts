import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereUniqueInput } from './filters-where-unique.input';

@ArgsType()
export class DeleteOneFiltersArgs {

    @Field(() => FiltersWhereUniqueInput, {nullable:false})
    where!: FiltersWhereUniqueInput;
}
