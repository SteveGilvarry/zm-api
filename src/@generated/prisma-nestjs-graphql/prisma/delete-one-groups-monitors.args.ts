import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsWhereUniqueInput } from '../groups-monitors/groups-monitors-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsWhereUniqueInput, {nullable:false})
    @Type(() => Groups_MonitorsWhereUniqueInput)
    where!: Groups_MonitorsWhereUniqueInput;
}
