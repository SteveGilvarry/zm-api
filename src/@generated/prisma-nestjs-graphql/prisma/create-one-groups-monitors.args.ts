import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsCreateInput } from '../groups-monitors/groups-monitors-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsCreateInput, {nullable:false})
    @Type(() => Groups_MonitorsCreateInput)
    data!: Groups_MonitorsCreateInput;
}
