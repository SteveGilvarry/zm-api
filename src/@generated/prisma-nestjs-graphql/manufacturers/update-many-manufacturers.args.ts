import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersUpdateManyMutationInput } from './manufacturers-update-many-mutation.input';
import { Type } from 'class-transformer';
import { ManufacturersWhereInput } from './manufacturers-where.input';

@ArgsType()
export class UpdateManyManufacturersArgs {

    @Field(() => ManufacturersUpdateManyMutationInput, {nullable:false})
    @Type(() => ManufacturersUpdateManyMutationInput)
    data!: ManufacturersUpdateManyMutationInput;

    @Field(() => ManufacturersWhereInput, {nullable:true})
    @Type(() => ManufacturersWhereInput)
    where?: ManufacturersWhereInput;
}
