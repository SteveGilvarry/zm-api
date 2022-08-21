import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedWhereInput } from '../events-archived/events-archived-where.input';
import { Type } from 'class-transformer';
import { Events_ArchivedOrderByWithRelationInput } from '../events-archived/events-archived-order-by-with-relation.input';
import { Events_ArchivedWhereUniqueInput } from '../events-archived/events-archived-where-unique.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class AggregateEventsArchivedArgs {

    @Field(() => Events_ArchivedWhereInput, {nullable:true})
    @Type(() => Events_ArchivedWhereInput)
    where?: Events_ArchivedWhereInput;

    @Field(() => [Events_ArchivedOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Events_ArchivedOrderByWithRelationInput>;

    @Field(() => Events_ArchivedWhereUniqueInput, {nullable:true})
    cursor?: Events_ArchivedWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
