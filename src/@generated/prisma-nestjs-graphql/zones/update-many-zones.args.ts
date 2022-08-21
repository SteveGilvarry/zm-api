import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesUpdateManyMutationInput } from './zones-update-many-mutation.input';
import { Type } from 'class-transformer';
import { ZonesWhereInput } from './zones-where.input';

@ArgsType()
export class UpdateManyZonesArgs {

    @Field(() => ZonesUpdateManyMutationInput, {nullable:false})
    @Type(() => ZonesUpdateManyMutationInput)
    data!: ZonesUpdateManyMutationInput;

    @Field(() => ZonesWhereInput, {nullable:true})
    @Type(() => ZonesWhereInput)
    where?: ZonesWhereInput;
}
