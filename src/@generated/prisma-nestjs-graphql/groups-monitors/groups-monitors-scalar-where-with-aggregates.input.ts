import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';

@InputType()
export class Groups_MonitorsScalarWhereWithAggregatesInput {

    @Field(() => [Groups_MonitorsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<Groups_MonitorsScalarWhereWithAggregatesInput>;

    @Field(() => [Groups_MonitorsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<Groups_MonitorsScalarWhereWithAggregatesInput>;

    @Field(() => [Groups_MonitorsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<Groups_MonitorsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    GroupId?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;
}
