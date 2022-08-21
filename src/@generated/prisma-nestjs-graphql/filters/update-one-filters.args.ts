import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersUpdateInput } from './filters-update.input';
import { Type } from 'class-transformer';
import { FiltersWhereUniqueInput } from './filters-where-unique.input';

@ArgsType()
export class UpdateOneFiltersArgs {

    @Field(() => FiltersUpdateInput, {nullable:false})
    @Type(() => FiltersUpdateInput)
    data!: FiltersUpdateInput;

    @Field(() => FiltersWhereUniqueInput, {nullable:false})
    @Type(() => FiltersWhereUniqueInput)
    where!: FiltersWhereUniqueInput;
}
