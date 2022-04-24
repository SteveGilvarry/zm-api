import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereUniqueInput } from './manufacturers-where-unique.input';

@ArgsType()
export class FindUniqueManufacturersArgs {

    @Field(() => ManufacturersWhereUniqueInput, {nullable:false})
    where!: ManufacturersWhereUniqueInput;
}
