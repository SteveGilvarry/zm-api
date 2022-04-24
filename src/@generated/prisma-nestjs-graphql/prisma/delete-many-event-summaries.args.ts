import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesWhereInput } from '../event-summaries/event-summaries-where.input';

@ArgsType()
export class DeleteManyEventSummariesArgs {

    @Field(() => Event_SummariesWhereInput, {nullable:true})
    where?: Event_SummariesWhereInput;
}
