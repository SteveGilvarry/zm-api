import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesWhereUniqueInput } from '../event-summaries/event-summaries-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneEventSummariesArgs {

    @Field(() => Event_SummariesWhereUniqueInput, {nullable:false})
    @Type(() => Event_SummariesWhereUniqueInput)
    where!: Event_SummariesWhereUniqueInput;
}
