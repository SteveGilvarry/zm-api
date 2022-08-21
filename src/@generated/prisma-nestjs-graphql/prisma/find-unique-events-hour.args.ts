import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourWhereUniqueInput } from '../events-hour/events-hour-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueEventsHourArgs {

    @Field(() => Events_HourWhereUniqueInput, {nullable:false})
    @Type(() => Events_HourWhereUniqueInput)
    where!: Events_HourWhereUniqueInput;
}
