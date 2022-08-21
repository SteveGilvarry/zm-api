import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesCreateInput } from '../event-summaries/event-summaries-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneEventSummariesArgs {

    @Field(() => Event_SummariesCreateInput, {nullable:false})
    @Type(() => Event_SummariesCreateInput)
    data!: Event_SummariesCreateInput;
}
