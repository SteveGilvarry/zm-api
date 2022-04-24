import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedCreateInput } from '../events-archived/events-archived-create.input';

@ArgsType()
export class CreateOneEventsArchivedArgs {

    @Field(() => Events_ArchivedCreateInput, {nullable:false})
    data!: Events_ArchivedCreateInput;
}
