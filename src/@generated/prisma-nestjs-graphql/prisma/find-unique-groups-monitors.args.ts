import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsWhereUniqueInput } from '../groups-monitors/groups-monitors-where-unique.input';

@ArgsType()
export class FindUniqueGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsWhereUniqueInput, {nullable:false})
    where!: Groups_MonitorsWhereUniqueInput;
}
