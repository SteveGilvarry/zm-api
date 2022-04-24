import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersUpdateManyMutationInput } from './manufacturers-update-many-mutation.input';
import { ManufacturersWhereInput } from './manufacturers-where.input';

@ArgsType()
export class UpdateManyManufacturersArgs {

    @Field(() => ManufacturersUpdateManyMutationInput, {nullable:false})
    data!: ManufacturersUpdateManyMutationInput;

    @Field(() => ManufacturersWhereInput, {nullable:true})
    where?: ManufacturersWhereInput;
}
