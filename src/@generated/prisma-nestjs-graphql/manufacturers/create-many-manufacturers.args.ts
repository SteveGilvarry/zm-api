import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersCreateManyInput } from './manufacturers-create-many.input';

@ArgsType()
export class CreateManyManufacturersArgs {

    @Field(() => [ManufacturersCreateManyInput], {nullable:false})
    data!: Array<ManufacturersCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
