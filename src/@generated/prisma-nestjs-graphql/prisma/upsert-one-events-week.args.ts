import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekWhereUniqueInput } from '../events-week/events-week-where-unique.input';
import { Type } from 'class-transformer';
import { Events_WeekCreateInput } from '../events-week/events-week-create.input';
import { Events_WeekUpdateInput } from '../events-week/events-week-update.input';

@ArgsType()
export class UpsertOneEventsWeekArgs {

    @Field(() => Events_WeekWhereUniqueInput, {nullable:false})
    @Type(() => Events_WeekWhereUniqueInput)
    where!: Events_WeekWhereUniqueInput;

    @Field(() => Events_WeekCreateInput, {nullable:false})
    @Type(() => Events_WeekCreateInput)
    create!: Events_WeekCreateInput;

    @Field(() => Events_WeekUpdateInput, {nullable:false})
    @Type(() => Events_WeekUpdateInput)
    update!: Events_WeekUpdateInput;
}
