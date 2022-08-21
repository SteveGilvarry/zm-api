import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedUpdateManyMutationInput } from '../events-archived/events-archived-update-many-mutation.input';
import { Type } from 'class-transformer';
import { Events_ArchivedWhereInput } from '../events-archived/events-archived-where.input';

@ArgsType()
export class UpdateManyEventsArchivedArgs {

    @Field(() => Events_ArchivedUpdateManyMutationInput, {nullable:false})
    @Type(() => Events_ArchivedUpdateManyMutationInput)
    data!: Events_ArchivedUpdateManyMutationInput;

    @Field(() => Events_ArchivedWhereInput, {nullable:true})
    @Type(() => Events_ArchivedWhereInput)
    where?: Events_ArchivedWhereInput;
}
