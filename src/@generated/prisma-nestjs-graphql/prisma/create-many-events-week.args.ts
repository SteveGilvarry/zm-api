import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekCreateManyInput } from '../events-week/events-week-create-many.input';

@ArgsType()
export class CreateManyEventsWeekArgs {

    @Field(() => [Events_WeekCreateManyInput], {nullable:false})
    data!: Array<Events_WeekCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
