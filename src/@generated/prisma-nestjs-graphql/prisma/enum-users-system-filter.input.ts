import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_System } from './users-system.enum';
import { NestedEnumUsers_SystemFilter } from './nested-enum-users-system-filter.input';

@InputType()
export class EnumUsers_SystemFilter {

    @Field(() => Users_System, {nullable:true})
    equals?: keyof typeof Users_System;

    @Field(() => [Users_System], {nullable:true})
    in?: Array<keyof typeof Users_System>;

    @Field(() => [Users_System], {nullable:true})
    notIn?: Array<keyof typeof Users_System>;

    @Field(() => NestedEnumUsers_SystemFilter, {nullable:true})
    not?: NestedEnumUsers_SystemFilter;
}
