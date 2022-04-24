import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Monitors } from './users-monitors.enum';
import { NestedEnumUsers_MonitorsFilter } from './nested-enum-users-monitors-filter.input';

@InputType()
export class EnumUsers_MonitorsFilter {

    @Field(() => Users_Monitors, {nullable:true})
    equals?: keyof typeof Users_Monitors;

    @Field(() => [Users_Monitors], {nullable:true})
    in?: Array<keyof typeof Users_Monitors>;

    @Field(() => [Users_Monitors], {nullable:true})
    notIn?: Array<keyof typeof Users_Monitors>;

    @Field(() => NestedEnumUsers_MonitorsFilter, {nullable:true})
    not?: NestedEnumUsers_MonitorsFilter;
}
