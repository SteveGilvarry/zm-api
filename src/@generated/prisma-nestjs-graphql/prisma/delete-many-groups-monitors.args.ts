import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsWhereInput } from '../groups-monitors/groups-monitors-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsWhereInput, {nullable:true})
    @Type(() => Groups_MonitorsWhereInput)
    where?: Groups_MonitorsWhereInput;
}
