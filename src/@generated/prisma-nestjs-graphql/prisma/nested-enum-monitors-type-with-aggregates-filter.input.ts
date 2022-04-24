import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Type } from './monitors-type.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumMonitors_TypeFilter } from './nested-enum-monitors-type-filter.input';

@InputType()
export class NestedEnumMonitors_TypeWithAggregatesFilter {

    @Field(() => Monitors_Type, {nullable:true})
    equals?: keyof typeof Monitors_Type;

    @Field(() => [Monitors_Type], {nullable:true})
    in?: Array<keyof typeof Monitors_Type>;

    @Field(() => [Monitors_Type], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Type>;

    @Field(() => NestedEnumMonitors_TypeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumMonitors_TypeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumMonitors_TypeFilter, {nullable:true})
    _min?: NestedEnumMonitors_TypeFilter;

    @Field(() => NestedEnumMonitors_TypeFilter, {nullable:true})
    _max?: NestedEnumMonitors_TypeFilter;
}
