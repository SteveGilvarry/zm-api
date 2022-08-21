import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayWhereInput } from '../events-day/events-day-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyEventsDayArgs {

    @Field(() => Events_DayWhereInput, {nullable:true})
    @Type(() => Events_DayWhereInput)
    where?: Events_DayWhereInput;
}
