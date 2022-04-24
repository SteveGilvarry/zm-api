import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { BigIntNullableWithAggregatesFilter } from '../prisma/big-int-nullable-with-aggregates-filter.input';

@InputType()
export class Events_ArchivedScalarWhereWithAggregatesInput {

    @Field(() => [Events_ArchivedScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<Events_ArchivedScalarWhereWithAggregatesInput>;

    @Field(() => [Events_ArchivedScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<Events_ArchivedScalarWhereWithAggregatesInput>;

    @Field(() => [Events_ArchivedScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<Events_ArchivedScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    EventId?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    DiskSpace?: BigIntNullableWithAggregatesFilter;
}
