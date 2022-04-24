import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereUniqueInput } from './events-where-unique.input';
import { EventsCreateInput } from './events-create.input';
import { EventsUpdateInput } from './events-update.input';

@ArgsType()
export class UpsertOneEventsArgs {

    @Field(() => EventsWhereUniqueInput, {nullable:false})
    where!: EventsWhereUniqueInput;

    @Field(() => EventsCreateInput, {nullable:false})
    create!: EventsCreateInput;

    @Field(() => EventsUpdateInput, {nullable:false})
    update!: EventsUpdateInput;
}
