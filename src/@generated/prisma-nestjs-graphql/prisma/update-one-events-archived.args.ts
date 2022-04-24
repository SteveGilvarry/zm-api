import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedUpdateInput } from '../events-archived/events-archived-update.input';
import { Events_ArchivedWhereUniqueInput } from '../events-archived/events-archived-where-unique.input';

@ArgsType()
export class UpdateOneEventsArchivedArgs {

    @Field(() => Events_ArchivedUpdateInput, {nullable:false})
    data!: Events_ArchivedUpdateInput;

    @Field(() => Events_ArchivedWhereUniqueInput, {nullable:false})
    where!: Events_ArchivedWhereUniqueInput;
}
