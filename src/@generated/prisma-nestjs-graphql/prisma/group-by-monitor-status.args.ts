import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereInput } from '../monitor-status/monitor-status-where.input';
import { Monitor_StatusOrderByWithAggregationInput } from '../monitor-status/monitor-status-order-by-with-aggregation.input';
import { Monitor_StatusScalarFieldEnum } from '../monitor-status/monitor-status-scalar-field.enum';
import { Monitor_StatusScalarWhereWithAggregatesInput } from '../monitor-status/monitor-status-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupByMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereInput, {nullable:true})
    where?: Monitor_StatusWhereInput;

    @Field(() => [Monitor_StatusOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Monitor_StatusOrderByWithAggregationInput>;

    @Field(() => [Monitor_StatusScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Monitor_StatusScalarFieldEnum>;

    @Field(() => Monitor_StatusScalarWhereWithAggregatesInput, {nullable:true})
    having?: Monitor_StatusScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
