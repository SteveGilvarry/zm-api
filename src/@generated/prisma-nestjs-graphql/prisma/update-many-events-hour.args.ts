import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourUpdateManyMutationInput } from '../events-hour/events-hour-update-many-mutation.input';
import { Events_HourWhereInput } from '../events-hour/events-hour-where.input';

@ArgsType()
export class UpdateManyEventsHourArgs {

    @Field(() => Events_HourUpdateManyMutationInput, {nullable:false})
    data!: Events_HourUpdateManyMutationInput;

    @Field(() => Events_HourWhereInput, {nullable:true})
    where?: Events_HourWhereInput;
}
