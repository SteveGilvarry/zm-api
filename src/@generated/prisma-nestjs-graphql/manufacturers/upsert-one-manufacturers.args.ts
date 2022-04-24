import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereUniqueInput } from './manufacturers-where-unique.input';
import { ManufacturersCreateInput } from './manufacturers-create.input';
import { ManufacturersUpdateInput } from './manufacturers-update.input';

@ArgsType()
export class UpsertOneManufacturersArgs {

    @Field(() => ManufacturersWhereUniqueInput, {nullable:false})
    where!: ManufacturersWhereUniqueInput;

    @Field(() => ManufacturersCreateInput, {nullable:false})
    create!: ManufacturersCreateInput;

    @Field(() => ManufacturersUpdateInput, {nullable:false})
    update!: ManufacturersUpdateInput;
}
