import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekWhereUniqueInput } from '../events-week/events-week-where-unique.input';

@ArgsType()
export class FindUniqueEventsWeekArgs {

    @Field(() => Events_WeekWhereUniqueInput, {nullable:false})
    where!: Events_WeekWhereUniqueInput;
}
