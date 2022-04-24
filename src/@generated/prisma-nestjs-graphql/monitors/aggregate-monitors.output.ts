import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { MonitorsCountAggregate } from './monitors-count-aggregate.output';
import { MonitorsAvgAggregate } from './monitors-avg-aggregate.output';
import { MonitorsSumAggregate } from './monitors-sum-aggregate.output';
import { MonitorsMinAggregate } from './monitors-min-aggregate.output';
import { MonitorsMaxAggregate } from './monitors-max-aggregate.output';

@ObjectType()
export class AggregateMonitors {

    @Field(() => MonitorsCountAggregate, {nullable:true})
    _count?: MonitorsCountAggregate;

    @Field(() => MonitorsAvgAggregate, {nullable:true})
    _avg?: MonitorsAvgAggregate;

    @Field(() => MonitorsSumAggregate, {nullable:true})
    _sum?: MonitorsSumAggregate;

    @Field(() => MonitorsMinAggregate, {nullable:true})
    _min?: MonitorsMinAggregate;

    @Field(() => MonitorsMaxAggregate, {nullable:true})
    _max?: MonitorsMaxAggregate;
}
