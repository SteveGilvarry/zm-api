import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { EnumMonitor_Status_StatusWithAggregatesFilter } from '../prisma/enum-monitor-status-status-with-aggregates-filter.input';
import { DecimalWithAggregatesFilter } from '../prisma/decimal-with-aggregates-filter.input';
import { BigIntNullableWithAggregatesFilter } from '../prisma/big-int-nullable-with-aggregates-filter.input';

@InputType()
export class Monitor_StatusScalarWhereWithAggregatesInput {

    @Field(() => [Monitor_StatusScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<Monitor_StatusScalarWhereWithAggregatesInput>;

    @Field(() => [Monitor_StatusScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<Monitor_StatusScalarWhereWithAggregatesInput>;

    @Field(() => [Monitor_StatusScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<Monitor_StatusScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => EnumMonitor_Status_StatusWithAggregatesFilter, {nullable:true})
    Status?: EnumMonitor_Status_StatusWithAggregatesFilter;

    @Field(() => DecimalWithAggregatesFilter, {nullable:true})
    CaptureFPS?: DecimalWithAggregatesFilter;

    @Field(() => DecimalWithAggregatesFilter, {nullable:true})
    AnalysisFPS?: DecimalWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    CaptureBandwidth?: IntWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    DayEventDiskSpace?: BigIntNullableWithAggregatesFilter;
}
