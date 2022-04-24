import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereInput } from './filters-where.input';
import { FiltersOrderByWithAggregationInput } from './filters-order-by-with-aggregation.input';
import { FiltersScalarFieldEnum } from './filters-scalar-field.enum';
import { FiltersScalarWhereWithAggregatesInput } from './filters-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { FiltersCountAggregateInput } from './filters-count-aggregate.input';
import { FiltersAvgAggregateInput } from './filters-avg-aggregate.input';
import { FiltersSumAggregateInput } from './filters-sum-aggregate.input';
import { FiltersMinAggregateInput } from './filters-min-aggregate.input';
import { FiltersMaxAggregateInput } from './filters-max-aggregate.input';

@ArgsType()
export class FiltersGroupByArgs {

    @Field(() => FiltersWhereInput, {nullable:true})
    where?: FiltersWhereInput;

    @Field(() => [FiltersOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<FiltersOrderByWithAggregationInput>;

    @Field(() => [FiltersScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof FiltersScalarFieldEnum>;

    @Field(() => FiltersScalarWhereWithAggregatesInput, {nullable:true})
    having?: FiltersScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => FiltersCountAggregateInput, {nullable:true})
    _count?: FiltersCountAggregateInput;

    @Field(() => FiltersAvgAggregateInput, {nullable:true})
    _avg?: FiltersAvgAggregateInput;

    @Field(() => FiltersSumAggregateInput, {nullable:true})
    _sum?: FiltersSumAggregateInput;

    @Field(() => FiltersMinAggregateInput, {nullable:true})
    _min?: FiltersMinAggregateInput;

    @Field(() => FiltersMaxAggregateInput, {nullable:true})
    _max?: FiltersMaxAggregateInput;
}
