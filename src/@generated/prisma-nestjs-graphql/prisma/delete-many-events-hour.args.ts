import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourWhereInput } from '../events-hour/events-hour-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyEventsHourArgs {

    @Field(() => Events_HourWhereInput, {nullable:true})
    @Type(() => Events_HourWhereInput)
    where?: Events_HourWhereInput;
}
