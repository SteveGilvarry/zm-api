import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Stream } from './users-stream.enum';
import { NestedEnumUsers_StreamWithAggregatesFilter } from './nested-enum-users-stream-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumUsers_StreamFilter } from './nested-enum-users-stream-filter.input';

@InputType()
export class EnumUsers_StreamWithAggregatesFilter {

    @Field(() => Users_Stream, {nullable:true})
    equals?: keyof typeof Users_Stream;

    @Field(() => [Users_Stream], {nullable:true})
    in?: Array<keyof typeof Users_Stream>;

    @Field(() => [Users_Stream], {nullable:true})
    notIn?: Array<keyof typeof Users_Stream>;

    @Field(() => NestedEnumUsers_StreamWithAggregatesFilter, {nullable:true})
    not?: NestedEnumUsers_StreamWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumUsers_StreamFilter, {nullable:true})
    _min?: NestedEnumUsers_StreamFilter;

    @Field(() => NestedEnumUsers_StreamFilter, {nullable:true})
    _max?: NestedEnumUsers_StreamFilter;
}
