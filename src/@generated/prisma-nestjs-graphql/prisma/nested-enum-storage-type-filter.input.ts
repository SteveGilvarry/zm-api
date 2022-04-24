import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Storage_Type } from './storage-type.enum';

@InputType()
export class NestedEnumStorage_TypeFilter {

    @Field(() => Storage_Type, {nullable:true})
    equals?: keyof typeof Storage_Type;

    @Field(() => [Storage_Type], {nullable:true})
    in?: Array<keyof typeof Storage_Type>;

    @Field(() => [Storage_Type], {nullable:true})
    notIn?: Array<keyof typeof Storage_Type>;

    @Field(() => NestedEnumStorage_TypeFilter, {nullable:true})
    not?: NestedEnumStorage_TypeFilter;
}
