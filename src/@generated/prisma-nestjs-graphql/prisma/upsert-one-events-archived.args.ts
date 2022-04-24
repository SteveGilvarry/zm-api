import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedWhereUniqueInput } from '../events-archived/events-archived-where-unique.input';
import { Events_ArchivedCreateInput } from '../events-archived/events-archived-create.input';
import { Events_ArchivedUpdateInput } from '../events-archived/events-archived-update.input';

@ArgsType()
export class UpsertOneEventsArchivedArgs {

    @Field(() => Events_ArchivedWhereUniqueInput, {nullable:false})
    where!: Events_ArchivedWhereUniqueInput;

    @Field(() => Events_ArchivedCreateInput, {nullable:false})
    create!: Events_ArchivedCreateInput;

    @Field(() => Events_ArchivedUpdateInput, {nullable:false})
    update!: Events_ArchivedUpdateInput;
}
