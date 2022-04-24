import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Type } from './monitors-type.enum';
import { NestedEnumMonitors_TypeFilter } from './nested-enum-monitors-type-filter.input';

@InputType()
export class EnumMonitors_TypeFilter {

    @Field(() => Monitors_Type, {nullable:true})
    equals?: keyof typeof Monitors_Type;

    @Field(() => [Monitors_Type], {nullable:true})
    in?: Array<keyof typeof Monitors_Type>;

    @Field(() => [Monitors_Type], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Type>;

    @Field(() => NestedEnumMonitors_TypeFilter, {nullable:true})
    not?: NestedEnumMonitors_TypeFilter;
}
