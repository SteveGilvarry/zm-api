import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereInput } from './manufacturers-where.input';

@ArgsType()
export class DeleteManyManufacturersArgs {

    @Field(() => ManufacturersWhereInput, {nullable:true})
    where?: ManufacturersWhereInput;
}
