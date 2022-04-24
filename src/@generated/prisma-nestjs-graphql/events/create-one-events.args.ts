import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsCreateInput } from './events-create.input';

@ArgsType()
export class CreateOneEventsArgs {

    @Field(() => EventsCreateInput, {nullable:false})
    data!: EventsCreateInput;
}
