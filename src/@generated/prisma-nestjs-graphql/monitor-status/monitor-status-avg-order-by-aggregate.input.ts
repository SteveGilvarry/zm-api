import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class Monitor_StatusAvgOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CaptureFPS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AnalysisFPS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CaptureBandwidth?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DayEventDiskSpace?: keyof typeof SortOrder;
}
