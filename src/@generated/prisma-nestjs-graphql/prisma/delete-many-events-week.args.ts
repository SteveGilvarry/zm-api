import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekWhereInput } from '../events-week/events-week-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyEventsWeekArgs {

    @Field(() => Events_WeekWhereInput, {nullable:true})
    @Type(() => Events_WeekWhereInput)
    where?: Events_WeekWhereInput;
}
