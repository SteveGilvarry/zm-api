import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Storage_Scheme } from './storage-scheme.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumStorage_SchemeFilter } from './nested-enum-storage-scheme-filter.input';

@InputType()
export class NestedEnumStorage_SchemeWithAggregatesFilter {

    @Field(() => Storage_Scheme, {nullable:true})
    equals?: keyof typeof Storage_Scheme;

    @Field(() => [Storage_Scheme], {nullable:true})
    in?: Array<keyof typeof Storage_Scheme>;

    @Field(() => [Storage_Scheme], {nullable:true})
    notIn?: Array<keyof typeof Storage_Scheme>;

    @Field(() => NestedEnumStorage_SchemeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumStorage_SchemeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumStorage_SchemeFilter, {nullable:true})
    _min?: NestedEnumStorage_SchemeFilter;

    @Field(() => NestedEnumStorage_SchemeFilter, {nullable:true})
    _max?: NestedEnumStorage_SchemeFilter;
}
