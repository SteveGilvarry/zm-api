import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersUpdateManyMutationInput } from './filters-update-many-mutation.input';
import { Type } from 'class-transformer';
import { FiltersWhereInput } from './filters-where.input';

@ArgsType()
export class UpdateManyFiltersArgs {

    @Field(() => FiltersUpdateManyMutationInput, {nullable:false})
    @Type(() => FiltersUpdateManyMutationInput)
    data!: FiltersUpdateManyMutationInput;

    @Field(() => FiltersWhereInput, {nullable:true})
    @Type(() => FiltersWhereInput)
    where?: FiltersWhereInput;
}
