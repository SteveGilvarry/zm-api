import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsCreateInput } from '../groups-monitors/groups-monitors-create.input';

@ArgsType()
export class CreateOneGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsCreateInput, {nullable:false})
    data!: Groups_MonitorsCreateInput;
}
