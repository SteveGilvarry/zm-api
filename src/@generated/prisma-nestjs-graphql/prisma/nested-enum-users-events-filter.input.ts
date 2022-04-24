import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Events } from './users-events.enum';

@InputType()
export class NestedEnumUsers_EventsFilter {

    @Field(() => Users_Events, {nullable:true})
    equals?: keyof typeof Users_Events;

    @Field(() => [Users_Events], {nullable:true})
    in?: Array<keyof typeof Users_Events>;

    @Field(() => [Users_Events], {nullable:true})
    notIn?: Array<keyof typeof Users_Events>;

    @Field(() => NestedEnumUsers_EventsFilter, {nullable:true})
    not?: NestedEnumUsers_EventsFilter;
}
