import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Events } from './users-events.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumUsers_EventsFilter } from './nested-enum-users-events-filter.input';

@InputType()
export class NestedEnumUsers_EventsWithAggregatesFilter {

    @Field(() => Users_Events, {nullable:true})
    equals?: keyof typeof Users_Events;

    @Field(() => [Users_Events], {nullable:true})
    in?: Array<keyof typeof Users_Events>;

    @Field(() => [Users_Events], {nullable:true})
    notIn?: Array<keyof typeof Users_Events>;

    @Field(() => NestedEnumUsers_EventsWithAggregatesFilter, {nullable:true})
    not?: NestedEnumUsers_EventsWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumUsers_EventsFilter, {nullable:true})
    _min?: NestedEnumUsers_EventsFilter;

    @Field(() => NestedEnumUsers_EventsFilter, {nullable:true})
    _max?: NestedEnumUsers_EventsFilter;
}
