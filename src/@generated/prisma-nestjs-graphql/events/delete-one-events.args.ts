import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereUniqueInput } from './events-where-unique.input';

@ArgsType()
export class DeleteOneEventsArgs {

    @Field(() => EventsWhereUniqueInput, {nullable:false})
    where!: EventsWhereUniqueInput;
}
