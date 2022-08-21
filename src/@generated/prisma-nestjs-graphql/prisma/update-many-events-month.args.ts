import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthUpdateManyMutationInput } from '../events-month/events-month-update-many-mutation.input';
import { Type } from 'class-transformer';
import { Events_MonthWhereInput } from '../events-month/events-month-where.input';

@ArgsType()
export class UpdateManyEventsMonthArgs {

    @Field(() => Events_MonthUpdateManyMutationInput, {nullable:false})
    @Type(() => Events_MonthUpdateManyMutationInput)
    data!: Events_MonthUpdateManyMutationInput;

    @Field(() => Events_MonthWhereInput, {nullable:true})
    @Type(() => Events_MonthWhereInput)
    where?: Events_MonthWhereInput;
}
