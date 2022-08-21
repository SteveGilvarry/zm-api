import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsWhereUniqueInput } from '../groups-monitors/groups-monitors-where-unique.input';
import { Type } from 'class-transformer';
import { Groups_MonitorsCreateInput } from '../groups-monitors/groups-monitors-create.input';
import { Groups_MonitorsUpdateInput } from '../groups-monitors/groups-monitors-update.input';

@ArgsType()
export class UpsertOneGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsWhereUniqueInput, {nullable:false})
    @Type(() => Groups_MonitorsWhereUniqueInput)
    where!: Groups_MonitorsWhereUniqueInput;

    @Field(() => Groups_MonitorsCreateInput, {nullable:false})
    @Type(() => Groups_MonitorsCreateInput)
    create!: Groups_MonitorsCreateInput;

    @Field(() => Groups_MonitorsUpdateInput, {nullable:false})
    @Type(() => Groups_MonitorsUpdateInput)
    update!: Groups_MonitorsUpdateInput;
}
