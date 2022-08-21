import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedWhereUniqueInput } from '../events-archived/events-archived-where-unique.input';
import { Type } from 'class-transformer';
import { Events_ArchivedCreateInput } from '../events-archived/events-archived-create.input';
import { Events_ArchivedUpdateInput } from '../events-archived/events-archived-update.input';

@ArgsType()
export class UpsertOneEventsArchivedArgs {

    @Field(() => Events_ArchivedWhereUniqueInput, {nullable:false})
    @Type(() => Events_ArchivedWhereUniqueInput)
    where!: Events_ArchivedWhereUniqueInput;

    @Field(() => Events_ArchivedCreateInput, {nullable:false})
    @Type(() => Events_ArchivedCreateInput)
    create!: Events_ArchivedCreateInput;

    @Field(() => Events_ArchivedUpdateInput, {nullable:false})
    @Type(() => Events_ArchivedUpdateInput)
    update!: Events_ArchivedUpdateInput;
}
