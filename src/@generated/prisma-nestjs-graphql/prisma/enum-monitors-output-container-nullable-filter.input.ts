import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_OutputContainer } from './monitors-output-container.enum';
import { NestedEnumMonitors_OutputContainerNullableFilter } from './nested-enum-monitors-output-container-nullable-filter.input';

@InputType()
export class EnumMonitors_OutputContainerNullableFilter {

    @Field(() => Monitors_OutputContainer, {nullable:true})
    equals?: keyof typeof Monitors_OutputContainer;

    @Field(() => [Monitors_OutputContainer], {nullable:true})
    in?: Array<keyof typeof Monitors_OutputContainer>;

    @Field(() => [Monitors_OutputContainer], {nullable:true})
    notIn?: Array<keyof typeof Monitors_OutputContainer>;

    @Field(() => NestedEnumMonitors_OutputContainerNullableFilter, {nullable:true})
    not?: NestedEnumMonitors_OutputContainerNullableFilter;
}
