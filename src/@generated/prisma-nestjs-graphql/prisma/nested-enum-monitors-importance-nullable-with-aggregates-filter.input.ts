import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Importance } from '../monitors/monitors-importance.enum';
import { NestedIntNullableFilter } from './nested-int-nullable-filter.input';
import { NestedEnumMonitors_ImportanceNullableFilter } from './nested-enum-monitors-importance-nullable-filter.input';

@InputType()
export class NestedEnumMonitors_ImportanceNullableWithAggregatesFilter {

    @Field(() => Monitors_Importance, {nullable:true})
    equals?: keyof typeof Monitors_Importance;

    @Field(() => [Monitors_Importance], {nullable:true})
    in?: Array<keyof typeof Monitors_Importance>;

    @Field(() => [Monitors_Importance], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Importance>;

    @Field(() => NestedEnumMonitors_ImportanceNullableWithAggregatesFilter, {nullable:true})
    not?: NestedEnumMonitors_ImportanceNullableWithAggregatesFilter;

    @Field(() => NestedIntNullableFilter, {nullable:true})
    _count?: NestedIntNullableFilter;

    @Field(() => NestedEnumMonitors_ImportanceNullableFilter, {nullable:true})
    _min?: NestedEnumMonitors_ImportanceNullableFilter;

    @Field(() => NestedEnumMonitors_ImportanceNullableFilter, {nullable:true})
    _max?: NestedEnumMonitors_ImportanceNullableFilter;
}
