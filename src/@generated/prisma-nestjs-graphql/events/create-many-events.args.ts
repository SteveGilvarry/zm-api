import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsCreateManyInput } from './events-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyEventsArgs {

    @Field(() => [EventsCreateManyInput], {nullable:false})
    @Type(() => EventsCreateManyInput)
    data!: Array<EventsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
