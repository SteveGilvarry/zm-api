import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersUpdateInput } from './manufacturers-update.input';
import { Type } from 'class-transformer';
import { ManufacturersWhereUniqueInput } from './manufacturers-where-unique.input';

@ArgsType()
export class UpdateOneManufacturersArgs {

    @Field(() => ManufacturersUpdateInput, {nullable:false})
    @Type(() => ManufacturersUpdateInput)
    data!: ManufacturersUpdateInput;

    @Field(() => ManufacturersWhereUniqueInput, {nullable:false})
    @Type(() => ManufacturersWhereUniqueInput)
    where!: ManufacturersWhereUniqueInput;
}
