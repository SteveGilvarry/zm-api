import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsWhereInput } from '../groups-monitors/groups-monitors-where.input';
import { Groups_MonitorsOrderByWithAggregationInput } from '../groups-monitors/groups-monitors-order-by-with-aggregation.input';
import { Groups_MonitorsScalarFieldEnum } from '../groups-monitors/groups-monitors-scalar-field.enum';
import { Groups_MonitorsScalarWhereWithAggregatesInput } from '../groups-monitors/groups-monitors-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupByGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsWhereInput, {nullable:true})
    where?: Groups_MonitorsWhereInput;

    @Field(() => [Groups_MonitorsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Groups_MonitorsOrderByWithAggregationInput>;

    @Field(() => [Groups_MonitorsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Groups_MonitorsScalarFieldEnum>;

    @Field(() => Groups_MonitorsScalarWhereWithAggregatesInput, {nullable:true})
    having?: Groups_MonitorsScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
