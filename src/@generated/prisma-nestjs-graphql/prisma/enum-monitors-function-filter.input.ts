import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Function } from './monitors-function.enum';
import { NestedEnumMonitors_FunctionFilter } from './nested-enum-monitors-function-filter.input';

@InputType()
export class EnumMonitors_FunctionFilter {

    @Field(() => Monitors_Function, {nullable:true})
    equals?: keyof typeof Monitors_Function;

    @Field(() => [Monitors_Function], {nullable:true})
    in?: Array<keyof typeof Monitors_Function>;

    @Field(() => [Monitors_Function], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Function>;

    @Field(() => NestedEnumMonitors_FunctionFilter, {nullable:true})
    not?: NestedEnumMonitors_FunctionFilter;
}
