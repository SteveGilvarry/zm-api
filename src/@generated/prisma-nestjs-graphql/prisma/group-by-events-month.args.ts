import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthWhereInput } from '../events-month/events-month-where.input';
import { Type } from 'class-transformer';
import { Events_MonthOrderByWithAggregationInput } from '../events-month/events-month-order-by-with-aggregation.input';
import { Events_MonthScalarFieldEnum } from '../events-month/events-month-scalar-field.enum';
import { Events_MonthScalarWhereWithAggregatesInput } from '../events-month/events-month-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupByEventsMonthArgs {

    @Field(() => Events_MonthWhereInput, {nullable:true})
    @Type(() => Events_MonthWhereInput)
    where?: Events_MonthWhereInput;

    @Field(() => [Events_MonthOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Events_MonthOrderByWithAggregationInput>;

    @Field(() => [Events_MonthScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Events_MonthScalarFieldEnum>;

    @Field(() => Events_MonthScalarWhereWithAggregatesInput, {nullable:true})
    having?: Events_MonthScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
