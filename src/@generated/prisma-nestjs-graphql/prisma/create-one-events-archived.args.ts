import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedCreateInput } from '../events-archived/events-archived-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneEventsArchivedArgs {

    @Field(() => Events_ArchivedCreateInput, {nullable:false})
    @Type(() => Events_ArchivedCreateInput)
    data!: Events_ArchivedCreateInput;
}
