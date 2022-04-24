import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { DateTimeNullableFilter } from '../prisma/date-time-nullable-filter.input';
import { BigIntNullableFilter } from '../prisma/big-int-nullable-filter.input';

@InputType()
export class Events_DayWhereInput {

    @Field(() => [Events_DayWhereInput], {nullable:true})
    AND?: Array<Events_DayWhereInput>;

    @Field(() => [Events_DayWhereInput], {nullable:true})
    OR?: Array<Events_DayWhereInput>;

    @Field(() => [Events_DayWhereInput], {nullable:true})
    NOT?: Array<Events_DayWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    EventId?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => DateTimeNullableFilter, {nullable:true})
    StartDateTime?: DateTimeNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    DiskSpace?: BigIntNullableFilter;
}
