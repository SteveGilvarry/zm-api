import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereUniqueInput } from './manufacturers-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneManufacturersArgs {

    @Field(() => ManufacturersWhereUniqueInput, {nullable:false})
    @Type(() => ManufacturersWhereUniqueInput)
    where!: ManufacturersWhereUniqueInput;
}
