import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthUpdateInput } from '../events-month/events-month-update.input';
import { Events_MonthWhereUniqueInput } from '../events-month/events-month-where-unique.input';

@ArgsType()
export class UpdateOneEventsMonthArgs {

    @Field(() => Events_MonthUpdateInput, {nullable:false})
    data!: Events_MonthUpdateInput;

    @Field(() => Events_MonthWhereUniqueInput, {nullable:false})
    where!: Events_MonthWhereUniqueInput;
}
