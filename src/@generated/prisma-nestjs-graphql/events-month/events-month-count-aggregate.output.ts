import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Events_MonthCountAggregate {

    @Field(() => Int, {nullable:false})
    EventId!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    StartDateTime!: number;

    @Field(() => Int, {nullable:false})
    DiskSpace!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
