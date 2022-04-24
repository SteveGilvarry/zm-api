import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Events_Scheme } from './events-scheme.enum';

@InputType()
export class NestedEnumEvents_SchemeFilter {

    @Field(() => Events_Scheme, {nullable:true})
    equals?: keyof typeof Events_Scheme;

    @Field(() => [Events_Scheme], {nullable:true})
    in?: Array<keyof typeof Events_Scheme>;

    @Field(() => [Events_Scheme], {nullable:true})
    notIn?: Array<keyof typeof Events_Scheme>;

    @Field(() => NestedEnumEvents_SchemeFilter, {nullable:true})
    not?: NestedEnumEvents_SchemeFilter;
}
