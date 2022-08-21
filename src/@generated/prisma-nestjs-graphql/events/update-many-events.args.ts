import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsUpdateManyMutationInput } from './events-update-many-mutation.input';
import { Type } from 'class-transformer';
import { EventsWhereInput } from './events-where.input';

@ArgsType()
export class UpdateManyEventsArgs {

    @Field(() => EventsUpdateManyMutationInput, {nullable:false})
    @Type(() => EventsUpdateManyMutationInput)
    data!: EventsUpdateManyMutationInput;

    @Field(() => EventsWhereInput, {nullable:true})
    @Type(() => EventsWhereInput)
    where?: EventsWhereInput;
}
