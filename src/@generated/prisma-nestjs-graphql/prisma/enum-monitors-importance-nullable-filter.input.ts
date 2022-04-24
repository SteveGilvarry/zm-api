import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Importance } from '../monitors/monitors-importance.enum';
import { NestedEnumMonitors_ImportanceNullableFilter } from './nested-enum-monitors-importance-nullable-filter.input';

@InputType()
export class EnumMonitors_ImportanceNullableFilter {

    @Field(() => Monitors_Importance, {nullable:true})
    equals?: keyof typeof Monitors_Importance;

    @Field(() => [Monitors_Importance], {nullable:true})
    in?: Array<keyof typeof Monitors_Importance>;

    @Field(() => [Monitors_Importance], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Importance>;

    @Field(() => NestedEnumMonitors_ImportanceNullableFilter, {nullable:true})
    not?: NestedEnumMonitors_ImportanceNullableFilter;
}
