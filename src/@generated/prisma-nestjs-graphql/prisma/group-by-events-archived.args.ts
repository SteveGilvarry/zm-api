import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_ArchivedWhereInput } from '../events-archived/events-archived-where.input';
import { Events_ArchivedOrderByWithAggregationInput } from '../events-archived/events-archived-order-by-with-aggregation.input';
import { Events_ArchivedScalarFieldEnum } from '../events-archived/events-archived-scalar-field.enum';
import { Events_ArchivedScalarWhereWithAggregatesInput } from '../events-archived/events-archived-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class GroupByEventsArchivedArgs {

    @Field(() => Events_ArchivedWhereInput, {nullable:true})
    where?: Events_ArchivedWhereInput;

    @Field(() => [Events_ArchivedOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<Events_ArchivedOrderByWithAggregationInput>;

    @Field(() => [Events_ArchivedScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof Events_ArchivedScalarFieldEnum>;

    @Field(() => Events_ArchivedScalarWhereWithAggregatesInput, {nullable:true})
    having?: Events_ArchivedScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
