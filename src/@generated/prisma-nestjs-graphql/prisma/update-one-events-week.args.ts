import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekUpdateInput } from '../events-week/events-week-update.input';
import { Events_WeekWhereUniqueInput } from '../events-week/events-week-where-unique.input';

@ArgsType()
export class UpdateOneEventsWeekArgs {

    @Field(() => Events_WeekUpdateInput, {nullable:false})
    data!: Events_WeekUpdateInput;

    @Field(() => Events_WeekWhereUniqueInput, {nullable:false})
    where!: Events_WeekWhereUniqueInput;
}
