import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class Monitor_StatusAvgAggregate {

    @Field(() => Float, {nullable:true})
    MonitorId?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    CaptureFPS?: any;

    @Field(() => GraphQLDecimal, {nullable:true})
    AnalysisFPS?: any;

    @Field(() => Float, {nullable:true})
    CaptureBandwidth?: number;

    @Field(() => Float, {nullable:true})
    DayEventDiskSpace?: number;
}
