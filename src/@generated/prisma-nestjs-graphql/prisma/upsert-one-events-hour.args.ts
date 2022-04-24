import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourWhereUniqueInput } from '../events-hour/events-hour-where-unique.input';
import { Events_HourCreateInput } from '../events-hour/events-hour-create.input';
import { Events_HourUpdateInput } from '../events-hour/events-hour-update.input';

@ArgsType()
export class UpsertOneEventsHourArgs {

    @Field(() => Events_HourWhereUniqueInput, {nullable:false})
    where!: Events_HourWhereUniqueInput;

    @Field(() => Events_HourCreateInput, {nullable:false})
    create!: Events_HourCreateInput;

    @Field(() => Events_HourUpdateInput, {nullable:false})
    update!: Events_HourUpdateInput;
}
