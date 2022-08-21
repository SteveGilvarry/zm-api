import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthWhereInput } from '../events-month/events-month-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyEventsMonthArgs {

    @Field(() => Events_MonthWhereInput, {nullable:true})
    @Type(() => Events_MonthWhereInput)
    where?: Events_MonthWhereInput;
}
