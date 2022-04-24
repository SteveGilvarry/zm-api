import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekWhereUniqueInput } from '../events-week/events-week-where-unique.input';
import { Events_WeekCreateInput } from '../events-week/events-week-create.input';
import { Events_WeekUpdateInput } from '../events-week/events-week-update.input';

@ArgsType()
export class UpsertOneEventsWeekArgs {

    @Field(() => Events_WeekWhereUniqueInput, {nullable:false})
    where!: Events_WeekWhereUniqueInput;

    @Field(() => Events_WeekCreateInput, {nullable:false})
    create!: Events_WeekCreateInput;

    @Field(() => Events_WeekUpdateInput, {nullable:false})
    update!: Events_WeekUpdateInput;
}
