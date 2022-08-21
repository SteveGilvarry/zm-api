import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekCreateInput } from '../events-week/events-week-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneEventsWeekArgs {

    @Field(() => Events_WeekCreateInput, {nullable:false})
    @Type(() => Events_WeekCreateInput)
    data!: Events_WeekCreateInput;
}
