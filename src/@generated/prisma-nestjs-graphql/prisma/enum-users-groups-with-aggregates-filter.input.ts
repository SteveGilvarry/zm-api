import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Groups } from './users-groups.enum';
import { NestedEnumUsers_GroupsWithAggregatesFilter } from './nested-enum-users-groups-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumUsers_GroupsFilter } from './nested-enum-users-groups-filter.input';

@InputType()
export class EnumUsers_GroupsWithAggregatesFilter {

    @Field(() => Users_Groups, {nullable:true})
    equals?: keyof typeof Users_Groups;

    @Field(() => [Users_Groups], {nullable:true})
    in?: Array<keyof typeof Users_Groups>;

    @Field(() => [Users_Groups], {nullable:true})
    notIn?: Array<keyof typeof Users_Groups>;

    @Field(() => NestedEnumUsers_GroupsWithAggregatesFilter, {nullable:true})
    not?: NestedEnumUsers_GroupsWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumUsers_GroupsFilter, {nullable:true})
    _min?: NestedEnumUsers_GroupsFilter;

    @Field(() => NestedEnumUsers_GroupsFilter, {nullable:true})
    _max?: NestedEnumUsers_GroupsFilter;
}
