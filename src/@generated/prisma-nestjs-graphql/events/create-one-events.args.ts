import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsCreateInput } from './events-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneEventsArgs {

    @Field(() => EventsCreateInput, {nullable:false})
    @Type(() => EventsCreateInput)
    data!: EventsCreateInput;
}
