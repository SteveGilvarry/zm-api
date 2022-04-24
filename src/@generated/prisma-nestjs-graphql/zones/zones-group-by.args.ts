import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesWhereInput } from './zones-where.input';
import { ZonesOrderByWithAggregationInput } from './zones-order-by-with-aggregation.input';
import { ZonesScalarFieldEnum } from './zones-scalar-field.enum';
import { ZonesScalarWhereWithAggregatesInput } from './zones-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { ZonesCountAggregateInput } from './zones-count-aggregate.input';
import { ZonesAvgAggregateInput } from './zones-avg-aggregate.input';
import { ZonesSumAggregateInput } from './zones-sum-aggregate.input';
import { ZonesMinAggregateInput } from './zones-min-aggregate.input';
import { ZonesMaxAggregateInput } from './zones-max-aggregate.input';

@ArgsType()
export class ZonesGroupByArgs {

    @Field(() => ZonesWhereInput, {nullable:true})
    where?: ZonesWhereInput;

    @Field(() => [ZonesOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<ZonesOrderByWithAggregationInput>;

    @Field(() => [ZonesScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof ZonesScalarFieldEnum>;

    @Field(() => ZonesScalarWhereWithAggregatesInput, {nullable:true})
    having?: ZonesScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ZonesCountAggregateInput, {nullable:true})
    _count?: ZonesCountAggregateInput;

    @Field(() => ZonesAvgAggregateInput, {nullable:true})
    _avg?: ZonesAvgAggregateInput;

    @Field(() => ZonesSumAggregateInput, {nullable:true})
    _sum?: ZonesSumAggregateInput;

    @Field(() => ZonesMinAggregateInput, {nullable:true})
    _min?: ZonesMinAggregateInput;

    @Field(() => ZonesMaxAggregateInput, {nullable:true})
    _max?: ZonesMaxAggregateInput;
}
