import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Event_SummariesWhereInput } from '../event-summaries/event-summaries-where.input';
import { Type } from 'class-transformer';
import { Event_SummariesOrderByWithRelationInput } from '../event-summaries/event-summaries-order-by-with-relation.input';
import { Event_SummariesWhereUniqueInput } from '../event-summaries/event-summaries-where-unique.input';
import { Int } from '@nestjs/graphql';
import { Event_SummariesScalarFieldEnum } from '../event-summaries/event-summaries-scalar-field.enum';

@ArgsType()
export class FindFirstEventSummariesArgs {

    @Field(() => Event_SummariesWhereInput, {nullable:true})
    @Type(() => Event_SummariesWhereInput)
    where?: Event_SummariesWhereInput;

    @Field(() => [Event_SummariesOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Event_SummariesOrderByWithRelationInput>;

    @Field(() => Event_SummariesWhereUniqueInput, {nullable:true})
    cursor?: Event_SummariesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [Event_SummariesScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof Event_SummariesScalarFieldEnum>;
}
