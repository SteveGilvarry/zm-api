import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ServersCountAggregate } from './servers-count-aggregate.output';
import { ServersAvgAggregate } from './servers-avg-aggregate.output';
import { ServersSumAggregate } from './servers-sum-aggregate.output';
import { ServersMinAggregate } from './servers-min-aggregate.output';
import { ServersMaxAggregate } from './servers-max-aggregate.output';

@ObjectType()
export class AggregateServers {

    @Field(() => ServersCountAggregate, {nullable:true})
    _count?: ServersCountAggregate;

    @Field(() => ServersAvgAggregate, {nullable:true})
    _avg?: ServersAvgAggregate;

    @Field(() => ServersSumAggregate, {nullable:true})
    _sum?: ServersSumAggregate;

    @Field(() => ServersMinAggregate, {nullable:true})
    _min?: ServersMinAggregate;

    @Field(() => ServersMaxAggregate, {nullable:true})
    _max?: ServersMaxAggregate;
}
