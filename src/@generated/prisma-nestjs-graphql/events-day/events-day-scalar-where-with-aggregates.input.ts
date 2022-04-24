import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { DateTimeNullableWithAggregatesFilter } from '../prisma/date-time-nullable-with-aggregates-filter.input';
import { BigIntNullableWithAggregatesFilter } from '../prisma/big-int-nullable-with-aggregates-filter.input';

@InputType()
export class Events_DayScalarWhereWithAggregatesInput {

    @Field(() => [Events_DayScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<Events_DayScalarWhereWithAggregatesInput>;

    @Field(() => [Events_DayScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<Events_DayScalarWhereWithAggregatesInput>;

    @Field(() => [Events_DayScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<Events_DayScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    EventId?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => DateTimeNullableWithAggregatesFilter, {nullable:true})
    StartDateTime?: DateTimeNullableWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    DiskSpace?: BigIntNullableWithAggregatesFilter;
}
