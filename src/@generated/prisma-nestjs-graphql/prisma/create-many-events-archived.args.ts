import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedCreateManyInput } from '../events-archived/events-archived-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyEventsArchivedArgs {

    @Field(() => [Events_ArchivedCreateManyInput], {nullable:false})
    @Type(() => Events_ArchivedCreateManyInput)
    data!: Array<Events_ArchivedCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
