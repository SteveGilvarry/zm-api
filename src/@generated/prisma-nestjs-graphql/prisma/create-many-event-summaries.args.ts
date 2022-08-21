import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesCreateManyInput } from '../event-summaries/event-summaries-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyEventSummariesArgs {

    @Field(() => [Event_SummariesCreateManyInput], {nullable:false})
    @Type(() => Event_SummariesCreateManyInput)
    data!: Array<Event_SummariesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
