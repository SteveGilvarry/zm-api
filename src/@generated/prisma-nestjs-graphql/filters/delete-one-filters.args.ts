import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereUniqueInput } from './filters-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneFiltersArgs {

    @Field(() => FiltersWhereUniqueInput, {nullable:false})
    @Type(() => FiltersWhereUniqueInput)
    where!: FiltersWhereUniqueInput;
}
