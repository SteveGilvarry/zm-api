import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereUniqueInput } from './manufacturers-where-unique.input';
import { Type } from 'class-transformer';
import { ManufacturersCreateInput } from './manufacturers-create.input';
import { ManufacturersUpdateInput } from './manufacturers-update.input';

@ArgsType()
export class UpsertOneManufacturersArgs {

    @Field(() => ManufacturersWhereUniqueInput, {nullable:false})
    @Type(() => ManufacturersWhereUniqueInput)
    where!: ManufacturersWhereUniqueInput;

    @Field(() => ManufacturersCreateInput, {nullable:false})
    @Type(() => ManufacturersCreateInput)
    create!: ManufacturersCreateInput;

    @Field(() => ManufacturersUpdateInput, {nullable:false})
    @Type(() => ManufacturersUpdateInput)
    update!: ManufacturersUpdateInput;
}
