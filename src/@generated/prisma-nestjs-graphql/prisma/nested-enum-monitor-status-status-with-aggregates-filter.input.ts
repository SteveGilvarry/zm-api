import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitor_Status_Status } from './monitor-status-status.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumMonitor_Status_StatusFilter } from './nested-enum-monitor-status-status-filter.input';

@InputType()
export class NestedEnumMonitor_Status_StatusWithAggregatesFilter {

    @Field(() => Monitor_Status_Status, {nullable:true})
    equals?: keyof typeof Monitor_Status_Status;

    @Field(() => [Monitor_Status_Status], {nullable:true})
    in?: Array<keyof typeof Monitor_Status_Status>;

    @Field(() => [Monitor_Status_Status], {nullable:true})
    notIn?: Array<keyof typeof Monitor_Status_Status>;

    @Field(() => NestedEnumMonitor_Status_StatusWithAggregatesFilter, {nullable:true})
    not?: NestedEnumMonitor_Status_StatusWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumMonitor_Status_StatusFilter, {nullable:true})
    _min?: NestedEnumMonitor_Status_StatusFilter;

    @Field(() => NestedEnumMonitor_Status_StatusFilter, {nullable:true})
    _max?: NestedEnumMonitor_Status_StatusFilter;
}
