import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsUpdateInput } from '../groups-monitors/groups-monitors-update.input';
import { Groups_MonitorsWhereUniqueInput } from '../groups-monitors/groups-monitors-where-unique.input';

@ArgsType()
export class UpdateOneGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsUpdateInput, {nullable:false})
    data!: Groups_MonitorsUpdateInput;

    @Field(() => Groups_MonitorsWhereUniqueInput, {nullable:false})
    where!: Groups_MonitorsWhereUniqueInput;
}
