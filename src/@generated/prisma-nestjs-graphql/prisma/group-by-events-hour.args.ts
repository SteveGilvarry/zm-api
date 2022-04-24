import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourWhereInput } from '../events-hour/events-hour-where.input';
import { Events_HourOrderByWithAggregationInput } from '../events-hour/events-hour-order-by-with-aggregation.input';
import { Events_HourScalarFieldEnum } from '../events-hour/events-hour-scalar-field.enum';
import { Events_HourScalarWhereWithAggregatesInput } from '../events-hour/events-hour-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupByEventsHourArgs {

    @Field(() => Events_HourWhereInput, {nullable:true})
    where?: Events_HourWhereInput;

    @Field(() => [Events_HourOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Events_HourOrderByWithAggregationInput>;

    @Field(() => [Events_HourScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Events_HourScalarFieldEnum>;

    @Field(() => Events_HourScalarWhereWithAggregatesInput, {nullable:true})
    having?: Events_HourScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
