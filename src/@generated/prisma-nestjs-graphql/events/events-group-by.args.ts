import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereInput } from './events-where.input';
import { Type } from 'class-transformer';
import { EventsOrderByWithAggregationInput } from './events-order-by-with-aggregation.input';
import { EventsScalarFieldEnum } from './events-scalar-field.enum';
import { EventsScalarWhereWithAggregatesInput } from './events-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { EventsCountAggregateInput } from './events-count-aggregate.input';
import { EventsAvgAggregateInput } from './events-avg-aggregate.input';
import { EventsSumAggregateInput } from './events-sum-aggregate.input';
import { EventsMinAggregateInput } from './events-min-aggregate.input';
import { EventsMaxAggregateInput } from './events-max-aggregate.input';

@ArgsType()
export class EventsGroupByArgs {

    @Field(() => EventsWhereInput, {nullable:true})
    @Type(() => EventsWhereInput)
    where?: EventsWhereInput;

    @Field(() => [EventsOrderByWithAggregationInput], {nullable:true})
    @Type(() => EventsOrderByWithAggregationInput)
    orderBy?: Array<EventsOrderByWithAggregationInput>;

    @Field(() => [EventsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof EventsScalarFieldEnum>;

    @Field(() => EventsScalarWhereWithAggregatesInput, {nullable:true})
    @Type(() => EventsScalarWhereWithAggregatesInput)
    having?: EventsScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => EventsCountAggregateInput, {nullable:true})
    @Type(() => EventsCountAggregateInput)
    _count?: EventsCountAggregateInput;

    @Field(() => EventsAvgAggregateInput, {nullable:true})
    @Type(() => EventsAvgAggregateInput)
    _avg?: EventsAvgAggregateInput;

    @Field(() => EventsSumAggregateInput, {nullable:true})
    @Type(() => EventsSumAggregateInput)
    _sum?: EventsSumAggregateInput;

    @Field(() => EventsMinAggregateInput, {nullable:true})
    @Type(() => EventsMinAggregateInput)
    _min?: EventsMinAggregateInput;

    @Field(() => EventsMaxAggregateInput, {nullable:true})
    @Type(() => EventsMaxAggregateInput)
    _max?: EventsMaxAggregateInput;
}
