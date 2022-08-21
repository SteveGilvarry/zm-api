import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthCreateInput } from '../events-month/events-month-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneEventsMonthArgs {

    @Field(() => Events_MonthCreateInput, {nullable:false})
    @Type(() => Events_MonthCreateInput)
    data!: Events_MonthCreateInput;
}
