import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesUpdateInput } from '../event-summaries/event-summaries-update.input';
import { Event_SummariesWhereUniqueInput } from '../event-summaries/event-summaries-where-unique.input';

@ArgsType()
export class UpdateOneEventSummariesArgs {

    @Field(() => Event_SummariesUpdateInput, {nullable:false})
    data!: Event_SummariesUpdateInput;

    @Field(() => Event_SummariesWhereUniqueInput, {nullable:false})
    where!: Event_SummariesWhereUniqueInput;
}
