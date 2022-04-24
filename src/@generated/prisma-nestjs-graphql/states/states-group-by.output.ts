import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { StatesCountAggregate } from './states-count-aggregate.output';
import { StatesAvgAggregate } from './states-avg-aggregate.output';
import { StatesSumAggregate } from './states-sum-aggregate.output';
import { StatesMinAggregate } from './states-min-aggregate.output';
import { StatesMaxAggregate } from './states-max-aggregate.output';

@ObjectType()
export class StatesGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => String, {nullable:false})
    Definition!: string;

    @Field(() => Int, {nullable:false})
    IsActive!: number;

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
