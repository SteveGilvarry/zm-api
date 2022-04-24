import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Event_Summaries {

    @Field(() => ID, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:true})
    TotalEvents!: number | null;

    @Field(() => String, {nullable:true})
    TotalEventDiskSpace!: bigint | null;

    @Field(() => Int, {nullable:true})
    HourEvents!: number | null;

    @Field(() => String, {nullable:true})
    HourEventDiskSpace!: bigint | null;

    @Field(() => Int, {nullable:true})
    DayEvents!: number | null;

    @Field(() => String, {nullable:true})
    DayEventDiskSpace!: bigint | null;

    @Field(() => Int, {nullable:true})
    WeekEvents!: number | null;

    @Field(() => String, {nullable:true})
    WeekEventDiskSpace!: bigint | null;

    @Field(() => Int, {nullable:true})
    MonthEvents!: number | null;

    @Field(() => String, {nullable:true})
    MonthEventDiskSpace!: bigint | null;

    @Field(() => Int, {nullable:true})
    ArchivedEvents!: number | null;

    @Field(() => String, {nullable:true})
    ArchivedEventDiskSpace!: bigint | null;
}
