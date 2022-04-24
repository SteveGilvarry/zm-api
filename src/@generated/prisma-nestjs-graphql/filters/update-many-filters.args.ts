import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersUpdateManyMutationInput } from './filters-update-many-mutation.input';
import { FiltersWhereInput } from './filters-where.input';

@ArgsType()
export class UpdateManyFiltersArgs {

    @Field(() => FiltersUpdateManyMutationInput, {nullable:false})
    data!: FiltersUpdateManyMutationInput;

    @Field(() => FiltersWhereInput, {nullable:true})
    where?: FiltersWhereInput;
}
