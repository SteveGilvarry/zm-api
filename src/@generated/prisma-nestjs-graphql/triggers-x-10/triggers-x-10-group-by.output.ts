import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { TriggersX10CountAggregate } from './triggers-x-10-count-aggregate.output';
import { TriggersX10AvgAggregate } from './triggers-x-10-avg-aggregate.output';
import { TriggersX10SumAggregate } from './triggers-x-10-sum-aggregate.output';
import { TriggersX10MinAggregate } from './triggers-x-10-min-aggregate.output';
import { TriggersX10MaxAggregate } from './triggers-x-10-max-aggregate.output';

@ObjectType()
export class TriggersX10GroupBy {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => String, {nullable:true})
    Activation?: string;

    @Field(() => String, {nullable:true})
    AlarmInput?: string;

    @Field(() => String, {nullable:true})
    AlarmOutput?: string;

    @Field(() => TriggersX10CountAggregate, {nullable:true})
    _count?: TriggersX10CountAggregate;

    @Field(() => TriggersX10AvgAggregate, {nullable:true})
    _avg?: TriggersX10AvgAggregate;

    @Field(() => TriggersX10SumAggregate, {nullable:true})
    _sum?: TriggersX10SumAggregate;

    @Field(() => TriggersX10MinAggregate, {nullable:true})
    _min?: TriggersX10MinAggregate;

    @Field(() => TriggersX10MaxAggregate, {nullable:true})
    _max?: TriggersX10MaxAggregate;
}
