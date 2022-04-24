import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class Event_SummariesMinAggregateInput {

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    TotalEvents?: true;

    @Field(() => Boolean, {nullable:true})
    TotalEventDiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    HourEvents?: true;

    @Field(() => Boolean, {nullable:true})
    HourEventDiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    DayEvents?: true;

    @Field(() => Boolean, {nullable:true})
    DayEventDiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    WeekEvents?: true;

    @Field(() => Boolean, {nullable:true})
    WeekEventDiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    MonthEvents?: true;

    @Field(() => Boolean, {nullable:true})
    MonthEventDiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    ArchivedEvents?: true;

    @Field(() => Boolean, {nullable:true})
    ArchivedEventDiskSpace?: true;
}
