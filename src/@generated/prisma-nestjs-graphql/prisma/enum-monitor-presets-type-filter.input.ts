import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { MonitorPresets_Type } from './monitor-presets-type.enum';
import { NestedEnumMonitorPresets_TypeFilter } from './nested-enum-monitor-presets-type-filter.input';

@InputType()
export class EnumMonitorPresets_TypeFilter {

    @Field(() => MonitorPresets_Type, {nullable:true})
    equals?: keyof typeof MonitorPresets_Type;

    @Field(() => [MonitorPresets_Type], {nullable:true})
    in?: Array<keyof typeof MonitorPresets_Type>;

    @Field(() => [MonitorPresets_Type], {nullable:true})
    notIn?: Array<keyof typeof MonitorPresets_Type>;

    @Field(() => NestedEnumMonitorPresets_TypeFilter, {nullable:true})
    not?: NestedEnumMonitorPresets_TypeFilter;
}
