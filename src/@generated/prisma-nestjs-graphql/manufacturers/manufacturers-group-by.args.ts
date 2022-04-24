import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereInput } from './manufacturers-where.input';
import { ManufacturersOrderByWithAggregationInput } from './manufacturers-order-by-with-aggregation.input';
import { ManufacturersScalarFieldEnum } from './manufacturers-scalar-field.enum';
import { ManufacturersScalarWhereWithAggregatesInput } from './manufacturers-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { ManufacturersCountAggregateInput } from './manufacturers-count-aggregate.input';
import { ManufacturersAvgAggregateInput } from './manufacturers-avg-aggregate.input';
import { ManufacturersSumAggregateInput } from './manufacturers-sum-aggregate.input';
import { ManufacturersMinAggregateInput } from './manufacturers-min-aggregate.input';
import { ManufacturersMaxAggregateInput } from './manufacturers-max-aggregate.input';

@ArgsType()
export class ManufacturersGroupByArgs {

    @Field(() => ManufacturersWhereInput, {nullable:true})
    where?: ManufacturersWhereInput;

    @Field(() => [ManufacturersOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<ManufacturersOrderByWithAggregationInput>;

    @Field(() => [ManufacturersScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof ManufacturersScalarFieldEnum>;

    @Field(() => ManufacturersScalarWhereWithAggregatesInput, {nullable:true})
    having?: ManufacturersScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ManufacturersCountAggregateInput, {nullable:true})
    _count?: ManufacturersCountAggregateInput;

    @Field(() => ManufacturersAvgAggregateInput, {nullable:true})
    _avg?: ManufacturersAvgAggregateInput;

    @Field(() => ManufacturersSumAggregateInput, {nullable:true})
    _sum?: ManufacturersSumAggregateInput;

    @Field(() => ManufacturersMinAggregateInput, {nullable:true})
    _min?: ManufacturersMinAggregateInput;

    @Field(() => ManufacturersMaxAggregateInput, {nullable:true})
    _max?: ManufacturersMaxAggregateInput;
}
