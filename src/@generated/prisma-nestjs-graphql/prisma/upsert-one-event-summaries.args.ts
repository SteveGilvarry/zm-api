import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesWhereUniqueInput } from '../event-summaries/event-summaries-where-unique.input';
import { Type } from 'class-transformer';
import { Event_SummariesCreateInput } from '../event-summaries/event-summaries-create.input';
import { Event_SummariesUpdateInput } from '../event-summaries/event-summaries-update.input';

@ArgsType()
export class UpsertOneEventSummariesArgs {

    @Field(() => Event_SummariesWhereUniqueInput, {nullable:false})
    @Type(() => Event_SummariesWhereUniqueInput)
    where!: Event_SummariesWhereUniqueInput;

    @Field(() => Event_SummariesCreateInput, {nullable:false})
    @Type(() => Event_SummariesCreateInput)
    create!: Event_SummariesCreateInput;

    @Field(() => Event_SummariesUpdateInput, {nullable:false})
    @Type(() => Event_SummariesUpdateInput)
    update!: Event_SummariesUpdateInput;
}
