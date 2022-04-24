import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthWhereUniqueInput } from '../events-month/events-month-where-unique.input';

@ArgsType()
export class FindUniqueEventsMonthArgs {

    @Field(() => Events_MonthWhereUniqueInput, {nullable:false})
    where!: Events_MonthWhereUniqueInput;
}
