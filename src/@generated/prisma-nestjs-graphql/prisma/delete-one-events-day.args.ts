import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayWhereUniqueInput } from '../events-day/events-day-where-unique.input';

@ArgsType()
export class DeleteOneEventsDayArgs {

    @Field(() => Events_DayWhereUniqueInput, {nullable:false})
    where!: Events_DayWhereUniqueInput;
}
