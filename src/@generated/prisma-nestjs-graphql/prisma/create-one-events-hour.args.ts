import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourCreateInput } from '../events-hour/events-hour-create.input';

@ArgsType()
export class CreateOneEventsHourArgs {

    @Field(() => Events_HourCreateInput, {nullable:false})
    data!: Events_HourCreateInput;
}
