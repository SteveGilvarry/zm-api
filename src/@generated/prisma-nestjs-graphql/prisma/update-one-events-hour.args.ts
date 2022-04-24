import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourUpdateInput } from '../events-hour/events-hour-update.input';
import { Events_HourWhereUniqueInput } from '../events-hour/events-hour-where-unique.input';

@ArgsType()
export class UpdateOneEventsHourArgs {

    @Field(() => Events_HourUpdateInput, {nullable:false})
    data!: Events_HourUpdateInput;

    @Field(() => Events_HourWhereUniqueInput, {nullable:false})
    where!: Events_HourWhereUniqueInput;
}
