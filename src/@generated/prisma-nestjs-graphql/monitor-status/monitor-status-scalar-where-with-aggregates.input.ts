import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Type } from 'class-transformer';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { EnumMonitor_Status_StatusWithAggregatesFilter } from '../prisma/enum-monitor-status-status-with-aggregates-filter.input';
import { DecimalWithAggregatesFilter } from '../prisma/decimal-with-aggregates-filter.input';
import { BigIntNullableWithAggregatesFilter } from '../prisma/big-int-nullable-with-aggregates-filter.input';

@InputType()
export class Monitor_StatusScalarWhereWithAggregatesInput {

    @Field(() => [Monitor_StatusScalarWhereWithAggregatesInput], {nullable:true})
    @Type(() => Monitor_StatusScalarWhereWithAggregatesInput)
    AND?: Array<Monitor_StatusScalarWhereWithAggregatesInput>;

    @Field(() => [Monitor_StatusScalarWhereWithAggregatesInput], {nullable:true})
    @Type(() => Monitor_StatusScalarWhereWithAggregatesInput)
    OR?: Array<Monitor_StatusScalarWhereWithAggregatesInput>;

    @Field(() => [Monitor_StatusScalarWhereWithAggregatesInput], {nullable:true})
    @Type(() => Monitor_StatusScalarWhereWithAggregatesInput)
    NOT?: Array<Monitor_StatusScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => EnumMonitor_Status_StatusWithAggregatesFilter, {nullable:true})
    Status?: EnumMonitor_Status_StatusWithAggregatesFilter;

    @Field(() => DecimalWithAggregatesFilter, {nullable:true})
    @Type(() => DecimalWithAggregatesFilter)
    CaptureFPS?: DecimalWithAggregatesFilter;

    @Field(() => DecimalWithAggregatesFilter, {nullable:true})
    @Type(() => DecimalWithAggregatesFilter)
    AnalysisFPS?: DecimalWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    CaptureBandwidth?: IntWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    DayEventDiskSpace?: BigIntNullableWithAggregatesFilter;
}
