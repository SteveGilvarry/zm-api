import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekUpdateManyMutationInput } from '../events-week/events-week-update-many-mutation.input';
import { Events_WeekWhereInput } from '../events-week/events-week-where.input';

@ArgsType()
export class UpdateManyEventsWeekArgs {

    @Field(() => Events_WeekUpdateManyMutationInput, {nullable:false})
    data!: Events_WeekUpdateManyMutationInput;

    @Field(() => Events_WeekWhereInput, {nullable:true})
    where?: Events_WeekWhereInput;
}
