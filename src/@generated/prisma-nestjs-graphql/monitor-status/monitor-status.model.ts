import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Monitor_Status_Status } from '../prisma/monitor-status-status.enum';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Decimal } from '@prisma/client/runtime';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Monitor_Status {

    @Field(() => ID, {nullable:false})
    MonitorId!: number;

    @Field(() => Monitor_Status_Status, {nullable:false,defaultValue:'Unknown'})
    Status!: keyof typeof Monitor_Status_Status;

    @Field(() => GraphQLDecimal, {nullable:false,defaultValue:0})
    CaptureFPS!: Decimal;

    @Field(() => GraphQLDecimal, {nullable:false,defaultValue:0})
    AnalysisFPS!: Decimal;

    @Field(() => Int, {nullable:false,defaultValue:0})
    CaptureBandwidth!: number;

    @Field(() => String, {nullable:true})
    DayEventDiskSpace!: bigint | null;
}
