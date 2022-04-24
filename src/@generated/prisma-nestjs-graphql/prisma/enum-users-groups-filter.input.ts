import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Groups } from './users-groups.enum';
import { NestedEnumUsers_GroupsFilter } from './nested-enum-users-groups-filter.input';

@InputType()
export class EnumUsers_GroupsFilter {

    @Field(() => Users_Groups, {nullable:true})
    equals?: keyof typeof Users_Groups;

    @Field(() => [Users_Groups], {nullable:true})
    in?: Array<keyof typeof Users_Groups>;

    @Field(() => [Users_Groups], {nullable:true})
    notIn?: Array<keyof typeof Users_Groups>;

    @Field(() => NestedEnumUsers_GroupsFilter, {nullable:true})
    not?: NestedEnumUsers_GroupsFilter;
}
