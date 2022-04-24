import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereInput } from './events-where.input';

@ArgsType()
export class DeleteManyEventsArgs {

    @Field(() => EventsWhereInput, {nullable:true})
    where?: EventsWhereInput;
}
