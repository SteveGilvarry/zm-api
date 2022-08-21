import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersCreateManyInput } from './manufacturers-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyManufacturersArgs {

    @Field(() => [ManufacturersCreateManyInput], {nullable:false})
    @Type(() => ManufacturersCreateManyInput)
    data!: Array<ManufacturersCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
