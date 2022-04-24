import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayWhereInput } from '../events-day/events-day-where.input';
import { Events_DayOrderByWithRelationInput } from '../events-day/events-day-order-by-with-relation.input';
import { Events_DayWhereUniqueInput } from '../events-day/events-day-where-unique.input';
import { Int } from '@nestjs/graphql';
import { Events_DayScalarFieldEnum } from '../events-day/events-day-scalar-field.enum';

@ArgsType()
export class FindFirstEventsDayArgs {

    @Field(() => Events_DayWhereInput, {nullable:true})
    where?: Events_DayWhereInput;

    @Field(() => [Events_DayOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Events_DayOrderByWithRelationInput>;

    @Field(() => Events_DayWhereUniqueInput, {nullable:true})
    cursor?: Events_DayWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [Events_DayScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof Events_DayScalarFieldEnum>;
}
