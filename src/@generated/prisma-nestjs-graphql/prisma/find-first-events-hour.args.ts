import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourWhereInput } from '../events-hour/events-hour-where.input';
import { Type } from 'class-transformer';
import { Events_HourOrderByWithRelationInput } from '../events-hour/events-hour-order-by-with-relation.input';
import { Events_HourWhereUniqueInput } from '../events-hour/events-hour-where-unique.input';
import { Int } from '@nestjs/graphql';
import { Events_HourScalarFieldEnum } from '../events-hour/events-hour-scalar-field.enum';

@ArgsType()
export class FindFirstEventsHourArgs {

    @Field(() => Events_HourWhereInput, {nullable:true})
    @Type(() => Events_HourWhereInput)
    where?: Events_HourWhereInput;

    @Field(() => [Events_HourOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Events_HourOrderByWithRelationInput>;

    @Field(() => Events_HourWhereUniqueInput, {nullable:true})
    cursor?: Events_HourWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [Events_HourScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof Events_HourScalarFieldEnum>;
}
