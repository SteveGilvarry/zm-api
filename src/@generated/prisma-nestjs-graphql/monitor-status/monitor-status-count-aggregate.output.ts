import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Monitor_StatusCountAggregate {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    Status!: number;

    @Field(() => Int, {nullable:false})
    CaptureFPS!: number;

    @Field(() => Int, {nullable:false})
    AnalysisFPS!: number;

    @Field(() => Int, {nullable:false})
    CaptureBandwidth!: number;

    @Field(() => Int, {nullable:false})
    DayEventDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
