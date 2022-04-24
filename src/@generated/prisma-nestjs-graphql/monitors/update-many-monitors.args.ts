import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsUpdateManyMutationInput } from './monitors-update-many-mutation.input';
import { MonitorsWhereInput } from './monitors-where.input';

@ArgsType()
export class UpdateManyMonitorsArgs {

    @Field(() => MonitorsUpdateManyMutationInput, {nullable:false})
    data!: MonitorsUpdateManyMutationInput;

    @Field(() => MonitorsWhereInput, {nullable:true})
    where?: MonitorsWhereInput;
}
