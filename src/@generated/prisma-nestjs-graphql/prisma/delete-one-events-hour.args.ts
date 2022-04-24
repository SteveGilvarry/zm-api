import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourWhereUniqueInput } from '../events-hour/events-hour-where-unique.input';

@ArgsType()
export class DeleteOneEventsHourArgs {

    @Field(() => Events_HourWhereUniqueInput, {nullable:false})
    where!: Events_HourWhereUniqueInput;
}
