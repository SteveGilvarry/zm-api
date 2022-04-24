import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayWhereUniqueInput } from '../events-day/events-day-where-unique.input';
import { Events_DayCreateInput } from '../events-day/events-day-create.input';
import { Events_DayUpdateInput } from '../events-day/events-day-update.input';

@ArgsType()
export class UpsertOneEventsDayArgs {

    @Field(() => Events_DayWhereUniqueInput, {nullable:false})
    where!: Events_DayWhereUniqueInput;

    @Field(() => Events_DayCreateInput, {nullable:false})
    create!: Events_DayCreateInput;

    @Field(() => Events_DayUpdateInput, {nullable:false})
    update!: Events_DayUpdateInput;
}
