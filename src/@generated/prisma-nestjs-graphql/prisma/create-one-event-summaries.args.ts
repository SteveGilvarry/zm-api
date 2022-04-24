import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesCreateInput } from '../event-summaries/event-summaries-create.input';

@ArgsType()
export class CreateOneEventSummariesArgs {

    @Field(() => Event_SummariesCreateInput, {nullable:false})
    data!: Event_SummariesCreateInput;
}
