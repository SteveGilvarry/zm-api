import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersCreateManyInput } from './filters-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyFiltersArgs {

    @Field(() => [FiltersCreateManyInput], {nullable:false})
    @Type(() => FiltersCreateManyInput)
    data!: Array<FiltersCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
