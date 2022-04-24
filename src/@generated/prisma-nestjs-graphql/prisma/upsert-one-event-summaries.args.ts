import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesWhereUniqueInput } from '../event-summaries/event-summaries-where-unique.input';
import { Event_SummariesCreateInput } from '../event-summaries/event-summaries-create.input';
import { Event_SummariesUpdateInput } from '../event-summaries/event-summaries-update.input';

@ArgsType()
export class UpsertOneEventSummariesArgs {

    @Field(() => Event_SummariesWhereUniqueInput, {nullable:false})
    where!: Event_SummariesWhereUniqueInput;

    @Field(() => Event_SummariesCreateInput, {nullable:false})
    create!: Event_SummariesCreateInput;

    @Field(() => Event_SummariesUpdateInput, {nullable:false})
    update!: Event_SummariesUpdateInput;
}
