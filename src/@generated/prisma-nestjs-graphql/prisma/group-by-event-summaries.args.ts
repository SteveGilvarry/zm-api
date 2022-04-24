import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesWhereInput } from '../event-summaries/event-summaries-where.input';
import { Event_SummariesOrderByWithAggregationInput } from '../event-summaries/event-summaries-order-by-with-aggregation.input';
import { Event_SummariesScalarFieldEnum } from '../event-summaries/event-summaries-scalar-field.enum';
import { Event_SummariesScalarWhereWithAggregatesInput } from '../event-summaries/event-summaries-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupByEventSummariesArgs {

    @Field(() => Event_SummariesWhereInput, {nullable:true})
    where?: Event_SummariesWhereInput;

    @Field(() => [Event_SummariesOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Event_SummariesOrderByWithAggregationInput>;

    @Field(() => [Event_SummariesScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Event_SummariesScalarFieldEnum>;

    @Field(() => Event_SummariesScalarWhereWithAggregatesInput, {nullable:true})
    having?: Event_SummariesScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
