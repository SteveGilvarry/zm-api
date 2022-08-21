import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereInput } from './filters-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyFiltersArgs {

    @Field(() => FiltersWhereInput, {nullable:true})
    @Type(() => FiltersWhereInput)
    where?: FiltersWhereInput;
}
