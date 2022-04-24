import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Events_MonthMaxAggregate {

    @Field(() => Int, {nullable:true})
    EventId?: number;

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => Date, {nullable:true})
    StartDateTime?: Date | string;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;
}
