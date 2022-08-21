import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereInput } from './events-where.input';
import { Type } from 'class-transformer';
import { EventsOrderByWithRelationInput } from './events-order-by-with-relation.input';
import { EventsWhereUniqueInput } from './events-where-unique.input';
import { Int } from '@nestjs/graphql';
import { EventsCountAggregateInput } from './events-count-aggregate.input';
import { EventsAvgAggregateInput } from './events-avg-aggregate.input';
import { EventsSumAggregateInput } from './events-sum-aggregate.input';
import { EventsMinAggregateInput } from './events-min-aggregate.input';
import { EventsMaxAggregateInput } from './events-max-aggregate.input';

@ArgsType()
export class EventsAggregateArgs {

    @Field(() => EventsWhereInput, {nullable:true})
    @Type(() => EventsWhereInput)
    where?: EventsWhereInput;

    @Field(() => [EventsOrderByWithRelationInput], {nullable:true})
    @Type(() => EventsOrderByWithRelationInput)
    orderBy?: Array<EventsOrderByWithRelationInput>;

    @Field(() => EventsWhereUniqueInput, {nullable:true})
    @Type(() => EventsWhereUniqueInput)
    cursor?: EventsWhereUniqueInput;

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
