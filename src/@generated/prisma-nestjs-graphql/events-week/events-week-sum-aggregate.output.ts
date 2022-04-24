import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Events_WeekSumAggregate {

    @Field(() => Int, {nullable:true})
    EventId?: number;

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;
}
