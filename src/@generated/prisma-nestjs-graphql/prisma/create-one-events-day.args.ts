import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayCreateInput } from '../events-day/events-day-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneEventsDayArgs {

    @Field(() => Events_DayCreateInput, {nullable:false})
    @Type(() => Events_DayCreateInput)
    data!: Events_DayCreateInput;
}
