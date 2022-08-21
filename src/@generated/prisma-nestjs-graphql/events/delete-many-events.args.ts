import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereInput } from './events-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyEventsArgs {

    @Field(() => EventsWhereInput, {nullable:true})
    @Type(() => EventsWhereInput)
    where?: EventsWhereInput;
}
