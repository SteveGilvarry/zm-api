import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayUpdateManyMutationInput } from '../events-day/events-day-update-many-mutation.input';
import { Events_DayWhereInput } from '../events-day/events-day-where.input';

@ArgsType()
export class UpdateManyEventsDayArgs {

    @Field(() => Events_DayUpdateManyMutationInput, {nullable:false})
    data!: Events_DayUpdateManyMutationInput;

    @Field(() => Events_DayWhereInput, {nullable:true})
    where?: Events_DayWhereInput;
}
