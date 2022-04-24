import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthWhereUniqueInput } from '../events-month/events-month-where-unique.input';
import { Events_MonthCreateInput } from '../events-month/events-month-create.input';
import { Events_MonthUpdateInput } from '../events-month/events-month-update.input';

@ArgsType()
export class UpsertOneEventsMonthArgs {

    @Field(() => Events_MonthWhereUniqueInput, {nullable:false})
    where!: Events_MonthWhereUniqueInput;

    @Field(() => Events_MonthCreateInput, {nullable:false})
    create!: Events_MonthCreateInput;

    @Field(() => Events_MonthUpdateInput, {nullable:false})
    update!: Events_MonthUpdateInput;
}
