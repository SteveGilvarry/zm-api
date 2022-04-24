import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedWhereUniqueInput } from '../events-archived/events-archived-where-unique.input';

@ArgsType()
export class DeleteOneEventsArchivedArgs {

    @Field(() => Events_ArchivedWhereUniqueInput, {nullable:false})
    where!: Events_ArchivedWhereUniqueInput;
}
