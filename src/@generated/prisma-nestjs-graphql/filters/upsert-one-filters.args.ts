import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereUniqueInput } from './filters-where-unique.input';
import { Type } from 'class-transformer';
import { FiltersCreateInput } from './filters-create.input';
import { FiltersUpdateInput } from './filters-update.input';

@ArgsType()
export class UpsertOneFiltersArgs {

    @Field(() => FiltersWhereUniqueInput, {nullable:false})
    @Type(() => FiltersWhereUniqueInput)
    where!: FiltersWhereUniqueInput;

    @Field(() => FiltersCreateInput, {nullable:false})
    @Type(() => FiltersCreateInput)
    create!: FiltersCreateInput;

    @Field(() => FiltersUpdateInput, {nullable:false})
    @Type(() => FiltersUpdateInput)
    update!: FiltersUpdateInput;
}
