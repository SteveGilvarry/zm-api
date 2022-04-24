import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthCreateInput } from '../events-month/events-month-create.input';

@ArgsType()
export class CreateOneEventsMonthArgs {

    @Field(() => Events_MonthCreateInput, {nullable:false})
    data!: Events_MonthCreateInput;
}
