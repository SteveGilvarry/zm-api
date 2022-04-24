import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedWhereInput } from '../events-archived/events-archived-where.input';

@ArgsType()
export class DeleteManyEventsArchivedArgs {

    @Field(() => Events_ArchivedWhereInput, {nullable:true})
    where?: Events_ArchivedWhereInput;
}
