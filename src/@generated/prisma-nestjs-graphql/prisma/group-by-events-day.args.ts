import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayWhereInput } from '../events-day/events-day-where.input';
import { Events_DayOrderByWithAggregationInput } from '../events-day/events-day-order-by-with-aggregation.input';
import { Events_DayScalarFieldEnum } from '../events-day/events-day-scalar-field.enum';
import { Events_DayScalarWhereWithAggregatesInput } from '../events-day/events-day-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupByEventsDayArgs {

    @Field(() => Events_DayWhereInput, {nullable:true})
    where?: Events_DayWhereInput;

    @Field(() => [Events_DayOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Events_DayOrderByWithAggregationInput>;

    @Field(() => [Events_DayScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Events_DayScalarFieldEnum>;

    @Field(() => Events_DayScalarWhereWithAggregatesInput, {nullable:true})
    having?: Events_DayScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
