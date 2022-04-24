import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_System } from './users-system.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumUsers_SystemFilter } from './nested-enum-users-system-filter.input';

@InputType()
export class NestedEnumUsers_SystemWithAggregatesFilter {

    @Field(() => Users_System, {nullable:true})
    equals?: keyof typeof Users_System;

    @Field(() => [Users_System], {nullable:true})
    in?: Array<keyof typeof Users_System>;

    @Field(() => [Users_System], {nullable:true})
    notIn?: Array<keyof typeof Users_System>;

    @Field(() => NestedEnumUsers_SystemWithAggregatesFilter, {nullable:true})
    not?: NestedEnumUsers_SystemWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumUsers_SystemFilter, {nullable:true})
    _min?: NestedEnumUsers_SystemFilter;

    @Field(() => NestedEnumUsers_SystemFilter, {nullable:true})
    _max?: NestedEnumUsers_SystemFilter;
}
