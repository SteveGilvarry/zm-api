import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Monitor_Status_Status } from '../prisma/monitor-status-status.enum';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@InputType()
export class Monitor_StatusCreateInput {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Monitor_Status_Status, {nullable:true})
    Status?: keyof typeof Monitor_Status_Status;

    @Field(() => GraphQLDecimal, {nullable:true})
    CaptureFPS?: any;

    @Field(() => GraphQLDecimal, {nullable:true})
    AnalysisFPS?: any;

    @Field(() => Int, {nullable:true})
    CaptureBandwidth?: number;

    @Field(() => String, {nullable:true})
    DayEventDiskSpace?: bigint | number;
}
