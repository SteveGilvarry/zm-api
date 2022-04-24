import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersCreateInput } from './manufacturers-create.input';

@ArgsType()
export class CreateOneManufacturersArgs {

    @Field(() => ManufacturersCreateInput, {nullable:false})
    data!: ManufacturersCreateInput;
}
