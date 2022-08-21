import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthWhereUniqueInput } from '../events-month/events-month-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneEventsMonthArgs {

    @Field(() => Events_MonthWhereUniqueInput, {nullable:false})
    @Type(() => Events_MonthWhereUniqueInput)
    where!: Events_MonthWhereUniqueInput;
}
