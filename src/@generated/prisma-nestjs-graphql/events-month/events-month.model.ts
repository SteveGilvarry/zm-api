import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Events_Month {

    @Field(() => ID, {nullable:false})
    EventId!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Date, {nullable:true})
    StartDateTime!: Date | null;

    @Field(() => String, {nullable:true})
    DiskSpace!: bigint | null;
}
