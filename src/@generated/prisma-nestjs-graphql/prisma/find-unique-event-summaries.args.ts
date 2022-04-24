import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesWhereUniqueInput } from '../event-summaries/event-summaries-where-unique.input';

@ArgsType()
export class FindUniqueEventSummariesArgs {

    @Field(() => Event_SummariesWhereUniqueInput, {nullable:false})
    where!: Event_SummariesWhereUniqueInput;
}
