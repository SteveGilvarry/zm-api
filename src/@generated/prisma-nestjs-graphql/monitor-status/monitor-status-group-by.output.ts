import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Monitor_Status_Status } from '../prisma/monitor-status-status.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Monitor_StatusCountAggregate } from './monitor-status-count-aggregate.output';
import { Monitor_StatusAvgAggregate } from './monitor-status-avg-aggregate.output';
import { Monitor_StatusSumAggregate } from './monitor-status-sum-aggregate.output';
import { Monitor_StatusMinAggregate } from './monitor-status-min-aggregate.output';
import { Monitor_StatusMaxAggregate } from './monitor-status-max-aggregate.output';

@ObjectType()
export class Monitor_StatusGroupBy {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Monitor_Status_Status, {nullable:false})
    Status!: keyof typeof Monitor_Status_Status;

    @Field(() => GraphQLDecimal, {nullable:false})
    CaptureFPS!: Decimal;

    @Field(() => GraphQLDecimal, {nullable:false})
    AnalysisFPS!: Decimal;

    @Field(() => Int, {nullable:false})
    CaptureBandwidth!: number;

    @Field(() => String, {nullable:true})
    DayEventDiskSpace?: bigint | number;

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
