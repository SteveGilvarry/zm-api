import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersCreateInput } from './filters-create.input';

@ArgsType()
export class CreateOneFiltersArgs {

    @Field(() => FiltersCreateInput, {nullable:false})
    data!: FiltersCreateInput;
}
