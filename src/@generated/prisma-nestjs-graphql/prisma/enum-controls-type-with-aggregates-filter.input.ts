import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Controls_Type } from './controls-type.enum';
import { NestedEnumControls_TypeWithAggregatesFilter } from './nested-enum-controls-type-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumControls_TypeFilter } from './nested-enum-controls-type-filter.input';

@InputType()
export class EnumControls_TypeWithAggregatesFilter {

    @Field(() => Controls_Type, {nullable:true})
    equals?: keyof typeof Controls_Type;

    @Field(() => [Controls_Type], {nullable:true})
    in?: Array<keyof typeof Controls_Type>;

    @Field(() => [Controls_Type], {nullable:true})
    notIn?: Array<keyof typeof Controls_Type>;

    @Field(() => NestedEnumControls_TypeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumControls_TypeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumControls_TypeFilter, {nullable:true})
    _min?: NestedEnumControls_TypeFilter;

    @Field(() => NestedEnumControls_TypeFilter, {nullable:true})
    _max?: NestedEnumControls_TypeFilter;
}
