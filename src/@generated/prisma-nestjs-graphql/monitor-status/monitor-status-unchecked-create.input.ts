import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Monitor_Status_Status } from '../prisma/monitor-status-status.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { transformToDecimal } from 'prisma-graphql-type-decimal';
import { Transform } from 'class-transformer';
import { Type } from 'class-transformer';

@InputType()
export class Monitor_StatusUncheckedCreateInput {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Monitor_Status_Status, {nullable:true})
    Status?: keyof typeof Monitor_Status_Status;

    @Field(() => GraphQLDecimal, {nullable:true})
    @Type(() => Object)
    @Transform(transformToDecimal)
    CaptureFPS?: Decimal;

    @Field(() => GraphQLDecimal, {nullable:true})
    @Type(() => Object)
    @Transform(transformToDecimal)
    AnalysisFPS?: Decimal;

    @Field(() => Int, {nullable:true})
    CaptureBandwidth?: number;

    @Field(() => String, {nullable:true})
    DayEventDiskSpace?: bigint | number;
}
