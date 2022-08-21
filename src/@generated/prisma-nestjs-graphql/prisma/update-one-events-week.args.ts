import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekUpdateInput } from '../events-week/events-week-update.input';
import { Type } from 'class-transformer';
import { Events_WeekWhereUniqueInput } from '../events-week/events-week-where-unique.input';

@ArgsType()
export class UpdateOneEventsWeekArgs {

    @Field(() => Events_WeekUpdateInput, {nullable:false})
    @Type(() => Events_WeekUpdateInput)
    data!: Events_WeekUpdateInput;

    @Field(() => Events_WeekWhereUniqueInput, {nullable:false})
    @Type(() => Events_WeekWhereUniqueInput)
    where!: Events_WeekWhereUniqueInput;
}
