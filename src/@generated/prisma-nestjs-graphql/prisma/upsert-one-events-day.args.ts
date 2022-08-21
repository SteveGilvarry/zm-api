import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayWhereUniqueInput } from '../events-day/events-day-where-unique.input';
import { Type } from 'class-transformer';
import { Events_DayCreateInput } from '../events-day/events-day-create.input';
import { Events_DayUpdateInput } from '../events-day/events-day-update.input';

@ArgsType()
export class UpsertOneEventsDayArgs {

    @Field(() => Events_DayWhereUniqueInput, {nullable:false})
    @Type(() => Events_DayWhereUniqueInput)
    where!: Events_DayWhereUniqueInput;

    @Field(() => Events_DayCreateInput, {nullable:false})
    @Type(() => Events_DayCreateInput)
    create!: Events_DayCreateInput;

    @Field(() => Events_DayUpdateInput, {nullable:false})
    @Type(() => Events_DayUpdateInput)
    update!: Events_DayUpdateInput;
}
