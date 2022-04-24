import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_OutputContainer } from './monitors-output-container.enum';
import { NestedIntNullableFilter } from './nested-int-nullable-filter.input';
import { NestedEnumMonitors_OutputContainerNullableFilter } from './nested-enum-monitors-output-container-nullable-filter.input';

@InputType()
export class NestedEnumMonitors_OutputContainerNullableWithAggregatesFilter {

    @Field(() => Monitors_OutputContainer, {nullable:true})
    equals?: keyof typeof Monitors_OutputContainer;

    @Field(() => [Monitors_OutputContainer], {nullable:true})
    in?: Array<keyof typeof Monitors_OutputContainer>;

    @Field(() => [Monitors_OutputContainer], {nullable:true})
    notIn?: Array<keyof typeof Monitors_OutputContainer>;

    @Field(() => NestedEnumMonitors_OutputContainerNullableWithAggregatesFilter, {nullable:true})
    not?: NestedEnumMonitors_OutputContainerNullableWithAggregatesFilter;

    @Field(() => NestedIntNullableFilter, {nullable:true})
    _count?: NestedIntNullableFilter;

    @Field(() => NestedEnumMonitors_OutputContainerNullableFilter, {nullable:true})
    _min?: NestedEnumMonitors_OutputContainerNullableFilter;

    @Field(() => NestedEnumMonitors_OutputContainerNullableFilter, {nullable:true})
    _max?: NestedEnumMonitors_OutputContainerNullableFilter;
}
