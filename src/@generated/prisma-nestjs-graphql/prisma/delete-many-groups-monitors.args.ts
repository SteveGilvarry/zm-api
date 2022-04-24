import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsWhereInput } from '../groups-monitors/groups-monitors-where.input';

@ArgsType()
export class DeleteManyGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsWhereInput, {nullable:true})
    where?: Groups_MonitorsWhereInput;
}
