import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Type } from './monitors-type.enum';

@InputType()
export class NestedEnumMonitors_TypeFilter {

    @Field(() => Monitors_Type, {nullable:true})
    equals?: keyof typeof Monitors_Type;

    @Field(() => [Monitors_Type], {nullable:true})
    in?: Array<keyof typeof Monitors_Type>;

    @Field(() => [Monitors_Type], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Type>;

    @Field(() => NestedEnumMonitors_TypeFilter, {nullable:true})
    not?: NestedEnumMonitors_TypeFilter;
}
