import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsUpdateManyMutationInput } from '../groups-monitors/groups-monitors-update-many-mutation.input';
import { Groups_MonitorsWhereInput } from '../groups-monitors/groups-monitors-where.input';

@ArgsType()
export class UpdateManyGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsUpdateManyMutationInput, {nullable:false})
    data!: Groups_MonitorsUpdateManyMutationInput;

    @Field(() => Groups_MonitorsWhereInput, {nullable:true})
    where?: Groups_MonitorsWhereInput;
}
