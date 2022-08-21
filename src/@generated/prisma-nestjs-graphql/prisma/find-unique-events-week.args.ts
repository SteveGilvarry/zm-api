import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekWhereUniqueInput } from '../events-week/events-week-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueEventsWeekArgs {

    @Field(() => Events_WeekWhereUniqueInput, {nullable:false})
    @Type(() => Events_WeekWhereUniqueInput)
    where!: Events_WeekWhereUniqueInput;
}
