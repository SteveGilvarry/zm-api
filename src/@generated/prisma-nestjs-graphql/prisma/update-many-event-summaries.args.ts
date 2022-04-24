import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesUpdateManyMutationInput } from '../event-summaries/event-summaries-update-many-mutation.input';
import { Event_SummariesWhereInput } from '../event-summaries/event-summaries-where.input';

@ArgsType()
export class UpdateManyEventSummariesArgs {

    @Field(() => Event_SummariesUpdateManyMutationInput, {nullable:false})
    data!: Event_SummariesUpdateManyMutationInput;

    @Field(() => Event_SummariesWhereInput, {nullable:true})
    where?: Event_SummariesWhereInput;
}
