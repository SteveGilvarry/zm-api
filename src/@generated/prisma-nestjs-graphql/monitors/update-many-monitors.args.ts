import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsUpdateManyMutationInput } from './monitors-update-many-mutation.input';
import { Type } from 'class-transformer';
import { MonitorsWhereInput } from './monitors-where.input';

@ArgsType()
export class UpdateManyMonitorsArgs {

    @Field(() => MonitorsUpdateManyMutationInput, {nullable:false})
    @Type(() => MonitorsUpdateManyMutationInput)
    data!: MonitorsUpdateManyMutationInput;

    @Field(() => MonitorsWhereInput, {nullable:true})
    @Type(() => MonitorsWhereInput)
    where?: MonitorsWhereInput;
}
