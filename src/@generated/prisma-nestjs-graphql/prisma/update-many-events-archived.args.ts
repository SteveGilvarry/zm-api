import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedUpdateManyMutationInput } from '../events-archived/events-archived-update-many-mutation.input';
import { Events_ArchivedWhereInput } from '../events-archived/events-archived-where.input';

@ArgsType()
export class UpdateManyEventsArchivedArgs {

    @Field(() => Events_ArchivedUpdateManyMutationInput, {nullable:false})
    data!: Events_ArchivedUpdateManyMutationInput;

    @Field(() => Events_ArchivedWhereInput, {nullable:true})
    where?: Events_ArchivedWhereInput;
}
