import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayCreateInput } from '../events-day/events-day-create.input';

@ArgsType()
export class CreateOneEventsDayArgs {

    @Field(() => Events_DayCreateInput, {nullable:false})
    data!: Events_DayCreateInput;
}
