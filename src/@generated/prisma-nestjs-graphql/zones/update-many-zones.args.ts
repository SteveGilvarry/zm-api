import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesUpdateManyMutationInput } from './zones-update-many-mutation.input';
import { ZonesWhereInput } from './zones-where.input';

@ArgsType()
export class UpdateManyZonesArgs {

    @Field(() => ZonesUpdateManyMutationInput, {nullable:false})
    data!: ZonesUpdateManyMutationInput;

    @Field(() => ZonesWhereInput, {nullable:true})
    where?: ZonesWhereInput;
}
