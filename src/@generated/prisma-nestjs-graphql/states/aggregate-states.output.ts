import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { StatesCountAggregate } from './states-count-aggregate.output';
import { StatesAvgAggregate } from './states-avg-aggregate.output';
import { StatesSumAggregate } from './states-sum-aggregate.output';
import { StatesMinAggregate } from './states-min-aggregate.output';
import { StatesMaxAggregate } from './states-max-aggregate.output';

@ObjectType()
export class AggregateStates {

    @Field(() => StatesCountAggregate, {nullable:true})
    _count?: StatesCountAggregate;

    @Field(() => StatesAvgAggregate, {nullable:true})
    _avg?: StatesAvgAggregate;

    @Field(() => StatesSumAggregate, {nullable:true})
    _sum?: StatesSumAggregate;

    @Field(() => StatesMinAggregate, {nullable:true})
    _min?: StatesMinAggregate;

    @Field(() => StatesMaxAggregate, {nullable:true})
    _max?: StatesMaxAggregate;
}
