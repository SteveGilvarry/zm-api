import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsUpdateInput } from './events-update.input';
import { EventsWhereUniqueInput } from './events-where-unique.input';

@ArgsType()
export class UpdateOneEventsArgs {

    @Field(() => EventsUpdateInput, {nullable:false})
    data!: EventsUpdateInput;

    @Field(() => EventsWhereUniqueInput, {nullable:false})
    where!: EventsWhereUniqueInput;
}
