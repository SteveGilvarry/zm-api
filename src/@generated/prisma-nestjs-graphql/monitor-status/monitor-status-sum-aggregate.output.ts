import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class Monitor_StatusSumAggregate {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    CaptureFPS?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    AnalysisFPS?: Decimal;

    @Field(() => Int, {nullable:true})
    CaptureBandwidth?: number;

    @Field(() => String, {nullable:true})
    DayEventDiskSpace?: bigint | number;
}
