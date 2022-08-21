import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereUniqueInput } from './events-where-unique.input';
import { Type } from 'class-transformer';
import { EventsCreateInput } from './events-create.input';
import { EventsUpdateInput } from './events-update.input';

@ArgsType()
export class UpsertOneEventsArgs {

    @Field(() => EventsWhereUniqueInput, {nullable:false})
    @Type(() => EventsWhereUniqueInput)
    where!: EventsWhereUniqueInput;

    @Field(() => EventsCreateInput, {nullable:false})
    @Type(() => EventsCreateInput)
    create!: EventsCreateInput;

    @Field(() => EventsUpdateInput, {nullable:false})
    @Type(() => EventsUpdateInput)
    update!: EventsUpdateInput;
}
