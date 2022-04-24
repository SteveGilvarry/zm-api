import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthUpdateManyMutationInput } from '../events-month/events-month-update-many-mutation.input';
import { Events_MonthWhereInput } from '../events-month/events-month-where.input';

@ArgsType()
export class UpdateManyEventsMonthArgs {

    @Field(() => Events_MonthUpdateManyMutationInput, {nullable:false})
    data!: Events_MonthUpdateManyMutationInput;

    @Field(() => Events_MonthWhereInput, {nullable:true})
    where?: Events_MonthWhereInput;
}
