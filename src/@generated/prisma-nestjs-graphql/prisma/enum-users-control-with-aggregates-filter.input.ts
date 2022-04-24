import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Control } from './users-control.enum';
import { NestedEnumUsers_ControlWithAggregatesFilter } from './nested-enum-users-control-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumUsers_ControlFilter } from './nested-enum-users-control-filter.input';

@InputType()
export class EnumUsers_ControlWithAggregatesFilter {

    @Field(() => Users_Control, {nullable:true})
    equals?: keyof typeof Users_Control;

    @Field(() => [Users_Control], {nullable:true})
    in?: Array<keyof typeof Users_Control>;

    @Field(() => [Users_Control], {nullable:true})
    notIn?: Array<keyof typeof Users_Control>;

    @Field(() => NestedEnumUsers_ControlWithAggregatesFilter, {nullable:true})
    not?: NestedEnumUsers_ControlWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumUsers_ControlFilter, {nullable:true})
    _min?: NestedEnumUsers_ControlFilter;

    @Field(() => NestedEnumUsers_ControlFilter, {nullable:true})
    _max?: NestedEnumUsers_ControlFilter;
}
