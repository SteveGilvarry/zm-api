import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Monitors } from './users-monitors.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumUsers_MonitorsFilter } from './nested-enum-users-monitors-filter.input';

@InputType()
export class NestedEnumUsers_MonitorsWithAggregatesFilter {

    @Field(() => Users_Monitors, {nullable:true})
    equals?: keyof typeof Users_Monitors;

    @Field(() => [Users_Monitors], {nullable:true})
    in?: Array<keyof typeof Users_Monitors>;

    @Field(() => [Users_Monitors], {nullable:true})
    notIn?: Array<keyof typeof Users_Monitors>;

    @Field(() => NestedEnumUsers_MonitorsWithAggregatesFilter, {nullable:true})
    not?: NestedEnumUsers_MonitorsWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumUsers_MonitorsFilter, {nullable:true})
    _min?: NestedEnumUsers_MonitorsFilter;

    @Field(() => NestedEnumUsers_MonitorsFilter, {nullable:true})
    _max?: NestedEnumUsers_MonitorsFilter;
}
