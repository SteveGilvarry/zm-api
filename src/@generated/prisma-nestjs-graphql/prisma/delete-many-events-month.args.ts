import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthWhereInput } from '../events-month/events-month-where.input';

@ArgsType()
export class DeleteManyEventsMonthArgs {

    @Field(() => Events_MonthWhereInput, {nullable:true})
    where?: Events_MonthWhereInput;
}
