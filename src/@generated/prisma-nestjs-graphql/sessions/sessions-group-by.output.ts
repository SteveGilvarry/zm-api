import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { SessionsCountAggregate } from './sessions-count-aggregate.output';
import { SessionsAvgAggregate } from './sessions-avg-aggregate.output';
import { SessionsSumAggregate } from './sessions-sum-aggregate.output';
import { SessionsMinAggregate } from './sessions-min-aggregate.output';
import { SessionsMaxAggregate } from './sessions-max-aggregate.output';

@ObjectType()
export class SessionsGroupBy {

    @Field(() => String, {nullable:false})
    id!: string;

    @Field(() => Int, {nullable:true})
    access?: number;

    @Field(() => String, {nullable:true})
    data?: string;

    @Field(() => SessionsCountAggregate, {nullable:true})
    _count?: SessionsCountAggregate;

    @Field(() => SessionsAvgAggregate, {nullable:true})
    _avg?: SessionsAvgAggregate;

    @Field(() => SessionsSumAggregate, {nullable:true})
    _sum?: SessionsSumAggregate;

    @Field(() => SessionsMinAggregate, {nullable:true})
    _min?: SessionsMinAggregate;

    @Field(() => SessionsMaxAggregate, {nullable:true})
    _max?: SessionsMaxAggregate;
}
