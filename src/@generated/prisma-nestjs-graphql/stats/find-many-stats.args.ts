import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsWhereInput } from './stats-where.input';
import { StatsOrderByWithRelationInput } from './stats-order-by-with-relation.input';
import { StatsWhereUniqueInput } from './stats-where-unique.input';
import { Int } from '@nestjs/graphql';
import { StatsScalarFieldEnum } from './stats-scalar-field.enum';

@ArgsType()
export class FindManyStatsArgs {

    @Field(() => StatsWhereInput, {nullable:true})
    where?: StatsWhereInput;

    @Field(() => [StatsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<StatsOrderByWithRelationInput>;

    @Field(() => StatsWhereUniqueInput, {nullable:true})
    cursor?: StatsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [StatsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof StatsScalarFieldEnum>;
}
