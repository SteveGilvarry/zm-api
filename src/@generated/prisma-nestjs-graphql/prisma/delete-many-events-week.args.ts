import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekWhereInput } from '../events-week/events-week-where.input';

@ArgsType()
export class DeleteManyEventsWeekArgs {

    @Field(() => Events_WeekWhereInput, {nullable:true})
    where?: Events_WeekWhereInput;
}
