import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Groups_MonitorsWhereInput } from '../groups-monitors/groups-monitors-where.input';
import { Type } from 'class-transformer';
import { Groups_MonitorsOrderByWithRelationInput } from '../groups-monitors/groups-monitors-order-by-with-relation.input';
import { Groups_MonitorsWhereUniqueInput } from '../groups-monitors/groups-monitors-where-unique.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class AggregateGroupsMonitorsArgs {

    @Field(() => Groups_MonitorsWhereInput, {nullable:true})
    @Type(() => Groups_MonitorsWhereInput)
    where?: Groups_MonitorsWhereInput;

    @Field(() => [Groups_MonitorsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Groups_MonitorsOrderByWithRelationInput>;

    @Field(() => Groups_MonitorsWhereUniqueInput, {nullable:true})
    cursor?: Groups_MonitorsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
