import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthUpdateInput } from '../events-month/events-month-update.input';
import { Type } from 'class-transformer';
import { Events_MonthWhereUniqueInput } from '../events-month/events-month-where-unique.input';

@ArgsType()
export class UpdateOneEventsMonthArgs {

    @Field(() => Events_MonthUpdateInput, {nullable:false})
    @Type(() => Events_MonthUpdateInput)
    data!: Events_MonthUpdateInput;

    @Field(() => Events_MonthWhereUniqueInput, {nullable:false})
    @Type(() => Events_MonthWhereUniqueInput)
    where!: Events_MonthWhereUniqueInput;
}
