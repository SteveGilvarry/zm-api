import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedWhereInput } from '../events-archived/events-archived-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyEventsArchivedArgs {

    @Field(() => Events_ArchivedWhereInput, {nullable:true})
    @Type(() => Events_ArchivedWhereInput)
    where?: Events_ArchivedWhereInput;
}
