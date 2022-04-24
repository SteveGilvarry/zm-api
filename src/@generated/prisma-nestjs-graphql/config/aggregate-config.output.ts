import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ConfigCountAggregate } from './config-count-aggregate.output';
import { ConfigAvgAggregate } from './config-avg-aggregate.output';
import { ConfigSumAggregate } from './config-sum-aggregate.output';
import { ConfigMinAggregate } from './config-min-aggregate.output';
import { ConfigMaxAggregate } from './config-max-aggregate.output';

@ObjectType()
export class AggregateConfig {

    @Field(() => ConfigCountAggregate, {nullable:true})
    _count?: ConfigCountAggregate;

    @Field(() => ConfigAvgAggregate, {nullable:true})
    _avg?: ConfigAvgAggregate;

    @Field(() => ConfigSumAggregate, {nullable:true})
    _sum?: ConfigSumAggregate;

    @Field(() => ConfigMinAggregate, {nullable:true})
    _min?: ConfigMinAggregate;

    @Field(() => ConfigMaxAggregate, {nullable:true})
    _max?: ConfigMaxAggregate;
}
