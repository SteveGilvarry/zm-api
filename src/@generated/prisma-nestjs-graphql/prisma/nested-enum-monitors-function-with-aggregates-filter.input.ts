import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Function } from './monitors-function.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumMonitors_FunctionFilter } from './nested-enum-monitors-function-filter.input';

@InputType()
export class NestedEnumMonitors_FunctionWithAggregatesFilter {

    @Field(() => Monitors_Function, {nullable:true})
    equals?: keyof typeof Monitors_Function;

    @Field(() => [Monitors_Function], {nullable:true})
    in?: Array<keyof typeof Monitors_Function>;

    @Field(() => [Monitors_Function], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Function>;

    @Field(() => NestedEnumMonitors_FunctionWithAggregatesFilter, {nullable:true})
    not?: NestedEnumMonitors_FunctionWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumMonitors_FunctionFilter, {nullable:true})
    _min?: NestedEnumMonitors_FunctionFilter;

    @Field(() => NestedEnumMonitors_FunctionFilter, {nullable:true})
    _max?: NestedEnumMonitors_FunctionFilter;
}
