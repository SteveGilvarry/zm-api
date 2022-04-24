import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersCreateManyInput } from './filters-create-many.input';

@ArgsType()
export class CreateManyFiltersArgs {

    @Field(() => [FiltersCreateManyInput], {nullable:false})
    data!: Array<FiltersCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
