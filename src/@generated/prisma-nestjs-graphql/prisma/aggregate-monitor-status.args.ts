import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereInput } from '../monitor-status/monitor-status-where.input';
import { Type } from 'class-transformer';
import { Monitor_StatusOrderByWithRelationInput } from '../monitor-status/monitor-status-order-by-with-relation.input';
import { Monitor_StatusWhereUniqueInput } from '../monitor-status/monitor-status-where-unique.input';
import { Int } from '@nestjs/graphql';

@ArgsType()
export class AggregateMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereInput, {nullable:true})
    @Type(() => Monitor_StatusWhereInput)
    where?: Monitor_StatusWhereInput;

    @Field(() => [Monitor_StatusOrderByWithRelationInput], {nullable:true})
    @Type(() => Monitor_StatusOrderByWithRelationInput)
    orderBy?: Array<Monitor_StatusOrderByWithRelationInput>;

    @Field(() => Monitor_StatusWhereUniqueInput, {nullable:true})
    @Type(() => Monitor_StatusWhereUniqueInput)
    cursor?: Monitor_StatusWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;
}
