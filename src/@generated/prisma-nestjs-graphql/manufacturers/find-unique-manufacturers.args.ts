import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereUniqueInput } from './manufacturers-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueManufacturersArgs {

    @Field(() => ManufacturersWhereUniqueInput, {nullable:false})
    @Type(() => ManufacturersWhereUniqueInput)
    where!: ManufacturersWhereUniqueInput;
}
