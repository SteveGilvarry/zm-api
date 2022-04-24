import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Storage_Type } from './storage-type.enum';
import { NestedEnumStorage_TypeWithAggregatesFilter } from './nested-enum-storage-type-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumStorage_TypeFilter } from './nested-enum-storage-type-filter.input';

@InputType()
export class EnumStorage_TypeWithAggregatesFilter {

    @Field(() => Storage_Type, {nullable:true})
    equals?: keyof typeof Storage_Type;

    @Field(() => [Storage_Type], {nullable:true})
    in?: Array<keyof typeof Storage_Type>;

    @Field(() => [Storage_Type], {nullable:true})
    notIn?: Array<keyof typeof Storage_Type>;

    @Field(() => NestedEnumStorage_TypeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumStorage_TypeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumStorage_TypeFilter, {nullable:true})
    _min?: NestedEnumStorage_TypeFilter;

    @Field(() => NestedEnumStorage_TypeFilter, {nullable:true})
    _max?: NestedEnumStorage_TypeFilter;
}
