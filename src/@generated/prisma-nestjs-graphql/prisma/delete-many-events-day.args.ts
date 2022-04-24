import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayWhereInput } from '../events-day/events-day-where.input';

@ArgsType()
export class DeleteManyEventsDayArgs {

    @Field(() => Events_DayWhereInput, {nullable:true})
    where?: Events_DayWhereInput;
}
