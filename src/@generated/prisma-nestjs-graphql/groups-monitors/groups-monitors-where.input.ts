import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';

@InputType()
export class Groups_MonitorsWhereInput {

    @Field(() => [Groups_MonitorsWhereInput], {nullable:true})
    AND?: Array<Groups_MonitorsWhereInput>;

    @Field(() => [Groups_MonitorsWhereInput], {nullable:true})
    OR?: Array<Groups_MonitorsWhereInput>;

    @Field(() => [Groups_MonitorsWhereInput], {nullable:true})
    NOT?: Array<Groups_MonitorsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    GroupId?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;
}
