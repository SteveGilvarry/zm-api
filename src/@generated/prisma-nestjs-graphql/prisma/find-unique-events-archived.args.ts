import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedWhereUniqueInput } from '../events-archived/events-archived-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueEventsArchivedArgs {

    @Field(() => Events_ArchivedWhereUniqueInput, {nullable:false})
    @Type(() => Events_ArchivedWhereUniqueInput)
    where!: Events_ArchivedWhereUniqueInput;
}
