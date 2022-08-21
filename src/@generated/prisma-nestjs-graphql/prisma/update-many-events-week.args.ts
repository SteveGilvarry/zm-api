import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekUpdateManyMutationInput } from '../events-week/events-week-update-many-mutation.input';
import { Type } from 'class-transformer';
import { Events_WeekWhereInput } from '../events-week/events-week-where.input';

@ArgsType()
export class UpdateManyEventsWeekArgs {

    @Field(() => Events_WeekUpdateManyMutationInput, {nullable:false})
    @Type(() => Events_WeekUpdateManyMutationInput)
    data!: Events_WeekUpdateManyMutationInput;

    @Field(() => Events_WeekWhereInput, {nullable:true})
    @Type(() => Events_WeekWhereInput)
    where?: Events_WeekWhereInput;
}
