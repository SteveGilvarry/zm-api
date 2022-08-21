import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsCreateManyInput } from '../groups-monitors/groups-monitors-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyGroupsMonitorsArgs {

    @Field(() => [Groups_MonitorsCreateManyInput], {nullable:false})
    @Type(() => Groups_MonitorsCreateManyInput)
    data!: Array<Groups_MonitorsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
