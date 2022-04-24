import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class Events_MonthCreateInput {

    @Field(() => Int, {nullable:false})
    EventId!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Date, {nullable:true})
    StartDateTime?: Date | string;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;
}
