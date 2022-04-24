import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { EnumMonitor_Status_StatusFilter } from '../prisma/enum-monitor-status-status-filter.input';
import { DecimalFilter } from '../prisma/decimal-filter.input';
import { BigIntNullableFilter } from '../prisma/big-int-nullable-filter.input';

@InputType()
export class Monitor_StatusWhereInput {

    @Field(() => [Monitor_StatusWhereInput], {nullable:true})
    AND?: Array<Monitor_StatusWhereInput>;

    @Field(() => [Monitor_StatusWhereInput], {nullable:true})
    OR?: Array<Monitor_StatusWhereInput>;

    @Field(() => [Monitor_StatusWhereInput], {nullable:true})
    NOT?: Array<Monitor_StatusWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => EnumMonitor_Status_StatusFilter, {nullable:true})
    Status?: EnumMonitor_Status_StatusFilter;

    @Field(() => DecimalFilter, {nullable:true})
    CaptureFPS?: DecimalFilter;

    @Field(() => DecimalFilter, {nullable:true})
    AnalysisFPS?: DecimalFilter;

    @Field(() => IntFilter, {nullable:true})
    CaptureBandwidth?: IntFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    DayEventDiskSpace?: BigIntNullableFilter;
}
