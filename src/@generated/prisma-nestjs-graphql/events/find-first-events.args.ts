import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { EventsWhereInput } from './events-where.input';
import { EventsOrderByWithRelationInput } from './events-order-by-with-relation.input';
import { EventsWhereUniqueInput } from './events-where-unique.input';
import { Int } from '@nestjs/graphql';
import { EventsScalarFieldEnum } from './events-scalar-field.enum';

@ArgsType()
export class FindFirstEventsArgs {

    @Field(() => EventsWhereInput, {nullable:true})
    where?: EventsWhereInput;

    @Field(() => [EventsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<EventsOrderByWithRelationInput>;

    @Field(() => EventsWhereUniqueInput, {nullable:true})
    cursor?: EventsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [EventsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof EventsScalarFieldEnum>;
}
