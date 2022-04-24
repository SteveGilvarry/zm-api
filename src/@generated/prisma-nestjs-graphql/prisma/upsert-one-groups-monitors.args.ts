import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsWhereUniqueInput } from '../groups-monitors/groups-monitors-where-unique.input';
import { Groups_MonitorsCreateInput } from '../groups-monitors/groups-monitors-create.input';
import { Groups_MonitorsUpdateInput } from '../groups-monitors/groups-monitors-update.input';

@ArgsType()
export class UpsertOneGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsWhereUniqueInput, {nullable:false})
    where!: Groups_MonitorsWhereUniqueInput;

    @Field(() => Groups_MonitorsCreateInput, {nullable:false})
    create!: Groups_MonitorsCreateInput;

    @Field(() => Groups_MonitorsUpdateInput, {nullable:false})
    update!: Groups_MonitorsUpdateInput;
}
