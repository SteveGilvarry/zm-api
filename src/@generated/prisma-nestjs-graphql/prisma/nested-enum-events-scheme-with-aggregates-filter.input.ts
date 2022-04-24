import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Events_Scheme } from './events-scheme.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumEvents_SchemeFilter } from './nested-enum-events-scheme-filter.input';

@InputType()
export class NestedEnumEvents_SchemeWithAggregatesFilter {

    @Field(() => Events_Scheme, {nullable:true})
    equals?: keyof typeof Events_Scheme;

    @Field(() => [Events_Scheme], {nullable:true})
    in?: Array<keyof typeof Events_Scheme>;

    @Field(() => [Events_Scheme], {nullable:true})
    notIn?: Array<keyof typeof Events_Scheme>;

    @Field(() => NestedEnumEvents_SchemeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumEvents_SchemeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumEvents_SchemeFilter, {nullable:true})
    _min?: NestedEnumEvents_SchemeFilter;

    @Field(() => NestedEnumEvents_SchemeFilter, {nullable:true})
    _max?: NestedEnumEvents_SchemeFilter;
}
