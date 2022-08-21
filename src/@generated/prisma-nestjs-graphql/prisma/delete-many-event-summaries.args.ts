import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesWhereInput } from '../event-summaries/event-summaries-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyEventSummariesArgs {

    @Field(() => Event_SummariesWhereInput, {nullable:true})
    @Type(() => Event_SummariesWhereInput)
    where?: Event_SummariesWhereInput;
}
