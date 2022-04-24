import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereInput } from './filters-where.input';

@ArgsType()
export class DeleteManyFiltersArgs {

    @Field(() => FiltersWhereInput, {nullable:true})
    where?: FiltersWhereInput;
}
