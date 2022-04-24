import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Monitor_StatusCountAggregate } from './monitor-status-count-aggregate.output';
import { Monitor_StatusAvgAggregate } from './monitor-status-avg-aggregate.output';
import { Monitor_StatusSumAggregate } from './monitor-status-sum-aggregate.output';
import { Monitor_StatusMinAggregate } from './monitor-status-min-aggregate.output';
import { Monitor_StatusMaxAggregate } from './monitor-status-max-aggregate.output';

@ObjectType()
export class AggregateMonitor_Status {

    @Field(() => Monitor_StatusCountAggregate, {nullable:true})
    _count?: Monitor_StatusCountAggregate;

    @Field(() => Monitor_StatusAvgAggregate, {nullable:true})
    _avg?: Monitor_StatusAvgAggregate;

    @Field(() => Monitor_StatusSumAggregate, {nullable:true})
    _sum?: Monitor_StatusSumAggregate;

    @Field(() => Monitor_StatusMinAggregate, {nullable:true})
    _min?: Monitor_StatusMinAggregate;

    @Field(() => Monitor_StatusMaxAggregate, {nullable:true})
    _max?: Monitor_StatusMaxAggregate;
}
