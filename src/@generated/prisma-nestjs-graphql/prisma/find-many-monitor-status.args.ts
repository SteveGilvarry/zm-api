import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereInput } from '../monitor-status/monitor-status-where.input';
import { Monitor_StatusOrderByWithRelationInput } from '../monitor-status/monitor-status-order-by-with-relation.input';
import { Monitor_StatusWhereUniqueInput } from '../monitor-status/monitor-status-where-unique.input';
import { Int } from '@nestjs/graphql';
import { Monitor_StatusScalarFieldEnum } from '../monitor-status/monitor-status-scalar-field.enum';

@ArgsType()
export class FindManyMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereInput, {nullable:true})
    where?: Monitor_StatusWhereInput;

    @Field(() => [Monitor_StatusOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Monitor_StatusOrderByWithRelationInput>;

    @Field(() => Monitor_StatusWhereUniqueInput, {nullable:true})
    cursor?: Monitor_StatusWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [Monitor_StatusScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof Monitor_StatusScalarFieldEnum>;
}
