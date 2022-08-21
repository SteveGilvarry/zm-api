import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereInput } from './manufacturers-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyManufacturersArgs {

    @Field(() => ManufacturersWhereInput, {nullable:true})
    @Type(() => ManufacturersWhereInput)
    where?: ManufacturersWhereInput;
}
