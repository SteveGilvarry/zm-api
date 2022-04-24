import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthWhereInput } from '../events-month/events-month-where.input';
import { Events_MonthOrderByWithRelationInput } from '../events-month/events-month-order-by-with-relation.input';
import { Events_MonthWhereUniqueInput } from '../events-month/events-month-where-unique.input';
import { Int } from '@nestjs/graphql';
import { Events_MonthScalarFieldEnum } from '../events-month/events-month-scalar-field.enum';

@ArgsType()
export class FindManyEventsMonthArgs {

    @Field(() => Events_MonthWhereInput, {nullable:true})
    where?: Events_MonthWhereInput;

    @Field(() => [Events_MonthOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Events_MonthOrderByWithRelationInput>;

    @Field(() => Events_MonthWhereUniqueInput, {nullable:true})
    cursor?: Events_MonthWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [Events_MonthScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof Events_MonthScalarFieldEnum>;
}
