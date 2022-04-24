import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Events_Scheme } from './events-scheme.enum';
import { NestedEnumEvents_SchemeFilter } from './nested-enum-events-scheme-filter.input';

@InputType()
export class EnumEvents_SchemeFilter {

    @Field(() => Events_Scheme, {nullable:true})
    equals?: keyof typeof Events_Scheme;

    @Field(() => [Events_Scheme], {nullable:true})
    in?: Array<keyof typeof Events_Scheme>;

    @Field(() => [Events_Scheme], {nullable:true})
    notIn?: Array<keyof typeof Events_Scheme>;

    @Field(() => NestedEnumEvents_SchemeFilter, {nullable:true})
    not?: NestedEnumEvents_SchemeFilter;
}
