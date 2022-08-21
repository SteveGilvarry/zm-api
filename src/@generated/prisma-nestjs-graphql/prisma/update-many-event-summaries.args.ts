import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesUpdateManyMutationInput } from '../event-summaries/event-summaries-update-many-mutation.input';
import { Type } from 'class-transformer';
import { Event_SummariesWhereInput } from '../event-summaries/event-summaries-where.input';

@ArgsType()
export class UpdateManyEventSummariesArgs {

    @Field(() => Event_SummariesUpdateManyMutationInput, {nullable:false})
    @Type(() => Event_SummariesUpdateManyMutationInput)
    data!: Event_SummariesUpdateManyMutationInput;

    @Field(() => Event_SummariesWhereInput, {nullable:true})
    @Type(() => Event_SummariesWhereInput)
    where?: Event_SummariesWhereInput;
}
