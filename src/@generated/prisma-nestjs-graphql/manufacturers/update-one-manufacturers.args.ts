import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersUpdateInput } from './manufacturers-update.input';
import { ManufacturersWhereUniqueInput } from './manufacturers-where-unique.input';

@ArgsType()
export class UpdateOneManufacturersArgs {

    @Field(() => ManufacturersUpdateInput, {nullable:false})
    data!: ManufacturersUpdateInput;

    @Field(() => ManufacturersWhereUniqueInput, {nullable:false})
    where!: ManufacturersWhereUniqueInput;
}
