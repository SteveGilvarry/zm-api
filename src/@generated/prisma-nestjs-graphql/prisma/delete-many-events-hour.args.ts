import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourWhereInput } from '../events-hour/events-hour-where.input';

@ArgsType()
export class DeleteManyEventsHourArgs {

    @Field(() => Events_HourWhereInput, {nullable:true})
    where?: Events_HourWhereInput;
}
