import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekWhereInput } from '../events-week/events-week-where.input';
import { Events_WeekOrderByWithAggregationInput } from '../events-week/events-week-order-by-with-aggregation.input';
import { Events_WeekScalarFieldEnum } from '../events-week/events-week-scalar-field.enum';
import { Events_WeekScalarWhereWithAggregatesInput } from '../events-week/events-week-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupByEventsWeekArgs {

    @Field(() => Events_WeekWhereInput, {nullable:true})
    where?: Events_WeekWhereInput;

    @Field(() => [Events_WeekOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Events_WeekOrderByWithAggregationInput>;

    @Field(() => [Events_WeekScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Events_WeekScalarFieldEnum>;

    @Field(() => Events_WeekScalarWhereWithAggregatesInput, {nullable:true})
    having?: Events_WeekScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
