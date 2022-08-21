import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedUpdateInput } from '../events-archived/events-archived-update.input';
import { Type } from 'class-transformer';
import { Events_ArchivedWhereUniqueInput } from '../events-archived/events-archived-where-unique.input';

@ArgsType()
export class UpdateOneEventsArchivedArgs {

    @Field(() => Events_ArchivedUpdateInput, {nullable:false})
    @Type(() => Events_ArchivedUpdateInput)
    data!: Events_ArchivedUpdateInput;

    @Field(() => Events_ArchivedWhereUniqueInput, {nullable:false})
    @Type(() => Events_ArchivedWhereUniqueInput)
    where!: Events_ArchivedWhereUniqueInput;
}
