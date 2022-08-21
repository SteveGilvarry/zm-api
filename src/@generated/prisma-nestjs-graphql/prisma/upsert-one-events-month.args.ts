import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthWhereUniqueInput } from '../events-month/events-month-where-unique.input';
import { Type } from 'class-transformer';
import { Events_MonthCreateInput } from '../events-month/events-month-create.input';
import { Events_MonthUpdateInput } from '../events-month/events-month-update.input';

@ArgsType()
export class UpsertOneEventsMonthArgs {

    @Field(() => Events_MonthWhereUniqueInput, {nullable:false})
    @Type(() => Events_MonthWhereUniqueInput)
    where!: Events_MonthWhereUniqueInput;

    @Field(() => Events_MonthCreateInput, {nullable:false})
    @Type(() => Events_MonthCreateInput)
    create!: Events_MonthCreateInput;

    @Field(() => Events_MonthUpdateInput, {nullable:false})
    @Type(() => Events_MonthUpdateInput)
    update!: Events_MonthUpdateInput;
}
