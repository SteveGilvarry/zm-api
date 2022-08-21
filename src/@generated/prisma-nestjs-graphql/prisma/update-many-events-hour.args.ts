import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourUpdateManyMutationInput } from '../events-hour/events-hour-update-many-mutation.input';
import { Type } from 'class-transformer';
import { Events_HourWhereInput } from '../events-hour/events-hour-where.input';

@ArgsType()
export class UpdateManyEventsHourArgs {

    @Field(() => Events_HourUpdateManyMutationInput, {nullable:false})
    @Type(() => Events_HourUpdateManyMutationInput)
    data!: Events_HourUpdateManyMutationInput;

    @Field(() => Events_HourWhereInput, {nullable:true})
    @Type(() => Events_HourWhereInput)
    where?: Events_HourWhereInput;
}
