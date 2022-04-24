import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekCreateInput } from '../events-week/events-week-create.input';

@ArgsType()
export class CreateOneEventsWeekArgs {

    @Field(() => Events_WeekCreateInput, {nullable:false})
    data!: Events_WeekCreateInput;
}
