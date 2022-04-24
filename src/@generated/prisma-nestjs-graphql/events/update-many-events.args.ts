import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsUpdateManyMutationInput } from './events-update-many-mutation.input';
import { EventsWhereInput } from './events-where.input';

@ArgsType()
export class UpdateManyEventsArgs {

    @Field(() => EventsUpdateManyMutationInput, {nullable:false})
    data!: EventsUpdateManyMutationInput;

    @Field(() => EventsWhereInput, {nullable:true})
    where?: EventsWhereInput;
}
