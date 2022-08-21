import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_WeekWhereInput } from '../events-week/events-week-where.input';
import { Type } from 'class-transformer';
import { Events_WeekOrderByWithRelationInput } from '../events-week/events-week-order-by-with-relation.input';
import { Events_WeekWhereUniqueInput } from '../events-week/events-week-where-unique.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class AggregateEventsWeekArgs {

    @Field(() => Events_WeekWhereInput, {nullable:true})
    @Type(() => Events_WeekWhereInput)
    where?: Events_WeekWhereInput;

    @Field(() => [Events_WeekOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Events_WeekOrderByWithRelationInput>;

    @Field(() => Events_WeekWhereUniqueInput, {nullable:true})
    cursor?: Events_WeekWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
