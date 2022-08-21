import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Monitor_Status_Status } from '../prisma/monitor-status-status.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class Monitor_StatusMinAggregate {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => Monitor_Status_Status, {nullable:true})
    Status?: keyof typeof Monitor_Status_Status;

    @Field(() => GraphQLDecimal, {nullable:true})
    CaptureFPS?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    AnalysisFPS?: Decimal;

    @Field(() => Int, {nullable:true})
    CaptureBandwidth?: number;

    @Field(() => String, {nullable:true})
    DayEventDiskSpace?: bigint | number;
}
