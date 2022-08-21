import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourCreateInput } from '../events-hour/events-hour-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneEventsHourArgs {

    @Field(() => Events_HourCreateInput, {nullable:false})
    @Type(() => Events_HourCreateInput)
    data!: Events_HourCreateInput;
}
