import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Importance } from '../monitors/monitors-importance.enum';

@InputType()
export class NestedEnumMonitors_ImportanceNullableFilter {

    @Field(() => Monitors_Importance, {nullable:true})
    equals?: keyof typeof Monitors_Importance;

    @Field(() => [Monitors_Importance], {nullable:true})
    in?: Array<keyof typeof Monitors_Importance>;

    @Field(() => [Monitors_Importance], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Importance>;

    @Field(() => NestedEnumMonitors_ImportanceNullableFilter, {nullable:true})
    not?: NestedEnumMonitors_ImportanceNullableFilter;
}
