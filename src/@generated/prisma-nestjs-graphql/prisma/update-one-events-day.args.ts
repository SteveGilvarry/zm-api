import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayUpdateInput } from '../events-day/events-day-update.input';
import { Type } from 'class-transformer';
import { Events_DayWhereUniqueInput } from '../events-day/events-day-where-unique.input';

@ArgsType()
export class UpdateOneEventsDayArgs {

    @Field(() => Events_DayUpdateInput, {nullable:false})
    @Type(() => Events_DayUpdateInput)
    data!: Events_DayUpdateInput;

    @Field(() => Events_DayWhereUniqueInput, {nullable:false})
    @Type(() => Events_DayWhereUniqueInput)
    where!: Events_DayWhereUniqueInput;
}
