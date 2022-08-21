import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersCreateInput } from './manufacturers-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneManufacturersArgs {

    @Field(() => ManufacturersCreateInput, {nullable:false})
    @Type(() => ManufacturersCreateInput)
    data!: ManufacturersCreateInput;
}
