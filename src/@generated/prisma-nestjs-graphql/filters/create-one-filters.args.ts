import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersCreateInput } from './filters-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneFiltersArgs {

    @Field(() => FiltersCreateInput, {nullable:false})
    @Type(() => FiltersCreateInput)
    data!: FiltersCreateInput;
}
