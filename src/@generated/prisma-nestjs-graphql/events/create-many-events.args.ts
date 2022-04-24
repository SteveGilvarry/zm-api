import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsCreateManyInput } from './events-create-many.input';

@ArgsType()
export class CreateManyEventsArgs {

    @Field(() => [EventsCreateManyInput], {nullable:false})
    data!: Array<EventsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
