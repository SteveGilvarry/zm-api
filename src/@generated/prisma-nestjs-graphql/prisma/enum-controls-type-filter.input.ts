import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Controls_Type } from './controls-type.enum';
import { NestedEnumControls_TypeFilter } from './nested-enum-controls-type-filter.input';

@InputType()
export class EnumControls_TypeFilter {

    @Field(() => Controls_Type, {nullable:true})
    equals?: keyof typeof Controls_Type;

    @Field(() => [Controls_Type], {nullable:true})
    in?: Array<keyof typeof Controls_Type>;

    @Field(() => [Controls_Type], {nullable:true})
    notIn?: Array<keyof typeof Controls_Type>;

    @Field(() => NestedEnumControls_TypeFilter, {nullable:true})
    not?: NestedEnumControls_TypeFilter;
}
