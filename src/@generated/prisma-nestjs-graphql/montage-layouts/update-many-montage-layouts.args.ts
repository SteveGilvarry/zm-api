import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsUpdateManyMutationInput } from './montage-layouts-update-many-mutation.input';
import { Type } from 'class-transformer';
import { MontageLayoutsWhereInput } from './montage-layouts-where.input';

@ArgsType()
export class UpdateManyMontageLayoutsArgs {

    @Field(() => MontageLayoutsUpdateManyMutationInput, {nullable:false})
    @Type(() => MontageLayoutsUpdateManyMutationInput)
    data!: MontageLayoutsUpdateManyMutationInput;

    @Field(() => MontageLayoutsWhereInput, {nullable:true})
    @Type(() => MontageLayoutsWhereInput)
    where?: MontageLayoutsWhereInput;
}
