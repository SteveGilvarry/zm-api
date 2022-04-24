import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Groups_MonitorsCountAggregate } from './groups-monitors-count-aggregate.output';
import { Groups_MonitorsAvgAggregate } from './groups-monitors-avg-aggregate.output';
import { Groups_MonitorsSumAggregate } from './groups-monitors-sum-aggregate.output';
import { Groups_MonitorsMinAggregate } from './groups-monitors-min-aggregate.output';
import { Groups_MonitorsMaxAggregate } from './groups-monitors-max-aggregate.output';

@ObjectType()
export class AggregateGroups_Monitors {

    @Field(() => Groups_MonitorsCountAggregate, {nullable:true})
    _count?: Groups_MonitorsCountAggregate;

    @Field(() => Groups_MonitorsAvgAggregate, {nullable:true})
    _avg?: Groups_MonitorsAvgAggregate;

    @Field(() => Groups_MonitorsSumAggregate, {nullable:true})
    _sum?: Groups_MonitorsSumAggregate;

    @Field(() => Groups_MonitorsMinAggregate, {nullable:true})
    _min?: Groups_MonitorsMinAggregate;

    @Field(() => Groups_MonitorsMaxAggregate, {nullable:true})
    _max?: Groups_MonitorsMaxAggregate;
}
