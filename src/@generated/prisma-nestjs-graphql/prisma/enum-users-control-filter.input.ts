import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Control } from './users-control.enum';
import { NestedEnumUsers_ControlFilter } from './nested-enum-users-control-filter.input';

@InputType()
export class EnumUsers_ControlFilter {

    @Field(() => Users_Control, {nullable:true})
    equals?: keyof typeof Users_Control;

    @Field(() => [Users_Control], {nullable:true})
    in?: Array<keyof typeof Users_Control>;

    @Field(() => [Users_Control], {nullable:true})
    notIn?: Array<keyof typeof Users_Control>;

    @Field(() => NestedEnumUsers_ControlFilter, {nullable:true})
    not?: NestedEnumUsers_ControlFilter;
}
