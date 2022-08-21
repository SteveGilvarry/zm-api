import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayWhereUniqueInput } from '../events-day/events-day-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueEventsDayArgs {

    @Field(() => Events_DayWhereUniqueInput, {nullable:false})
    @Type(() => Events_DayWhereUniqueInput)
    where!: Events_DayWhereUniqueInput;
}
