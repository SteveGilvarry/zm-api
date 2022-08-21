import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekCreateManyInput } from '../events-week/events-week-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyEventsWeekArgs {

    @Field(() => [Events_WeekCreateManyInput], {nullable:false})
    @Type(() => Events_WeekCreateManyInput)
    data!: Array<Events_WeekCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
