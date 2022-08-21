import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsUpdateInput } from './events-update.input';
import { Type } from 'class-transformer';
import { EventsWhereUniqueInput } from './events-where-unique.input';

@ArgsType()
export class UpdateOneEventsArgs {

    @Field(() => EventsUpdateInput, {nullable:false})
    @Type(() => EventsUpdateInput)
    data!: EventsUpdateInput;

    @Field(() => EventsWhereUniqueInput, {nullable:false})
    @Type(() => EventsWhereUniqueInput)
    where!: EventsWhereUniqueInput;
}
