import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereUniqueInput } from './events-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneEventsArgs {

    @Field(() => EventsWhereUniqueInput, {nullable:false})
    @Type(() => EventsWhereUniqueInput)
    where!: EventsWhereUniqueInput;
}
