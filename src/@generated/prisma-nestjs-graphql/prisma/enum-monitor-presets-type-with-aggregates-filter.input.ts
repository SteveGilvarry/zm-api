import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { MonitorPresets_Type } from './monitor-presets-type.enum';
import { NestedEnumMonitorPresets_TypeWithAggregatesFilter } from './nested-enum-monitor-presets-type-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumMonitorPresets_TypeFilter } from './nested-enum-monitor-presets-type-filter.input';

@InputType()
export class EnumMonitorPresets_TypeWithAggregatesFilter {

    @Field(() => MonitorPresets_Type, {nullable:true})
    equals?: keyof typeof MonitorPresets_Type;

    @Field(() => [MonitorPresets_Type], {nullable:true})
    in?: Array<keyof typeof MonitorPresets_Type>;

    @Field(() => [MonitorPresets_Type], {nullable:true})
    notIn?: Array<keyof typeof MonitorPresets_Type>;

    @Field(() => NestedEnumMonitorPresets_TypeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumMonitorPresets_TypeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumMonitorPresets_TypeFilter, {nullable:true})
    _min?: NestedEnumMonitorPresets_TypeFilter;

    @Field(() => NestedEnumMonitorPresets_TypeFilter, {nullable:true})
    _max?: NestedEnumMonitorPresets_TypeFilter;
}
