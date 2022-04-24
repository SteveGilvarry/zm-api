import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Storage_Scheme } from './storage-scheme.enum';

@InputType()
export class NestedEnumStorage_SchemeFilter {

    @Field(() => Storage_Scheme, {nullable:true})
    equals?: keyof typeof Storage_Scheme;

    @Field(() => [Storage_Scheme], {nullable:true})
    in?: Array<keyof typeof Storage_Scheme>;

    @Field(() => [Storage_Scheme], {nullable:true})
    notIn?: Array<keyof typeof Storage_Scheme>;

    @Field(() => NestedEnumStorage_SchemeFilter, {nullable:true})
    not?: NestedEnumStorage_SchemeFilter;
}
