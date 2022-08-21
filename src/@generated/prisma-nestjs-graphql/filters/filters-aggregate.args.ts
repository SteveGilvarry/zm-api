import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FiltersWhereInput } from './filters-where.input';
import { Type } from 'class-transformer';
import { FiltersOrderByWithRelationInput } from './filters-order-by-with-relation.input';
import { FiltersWhereUniqueInput } from './filters-where-unique.input';
import { Int } from '@nestjs/graphql';
import { FiltersCountAggregateInput } from './filters-count-aggregate.input';
import { FiltersAvgAggregateInput } from './filters-avg-aggregate.input';
import { FiltersSumAggregateInput } from './filters-sum-aggregate.input';
import { FiltersMinAggregateInput } from './filters-min-aggregate.input';
import { FiltersMaxAggregateInput } from './filters-max-aggregate.input';

@ArgsType()
export class FiltersAggregateArgs {

    @Field(() => FiltersWhereInput, {nullable:true})
    @Type(() => FiltersWhereInput)
    where?: FiltersWhereInput;

    @Field(() => [FiltersOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<FiltersOrderByWithRelationInput>;

    @Field(() => FiltersWhereUniqueInput, {nullable:true})
    cursor?: FiltersWhereUniqueInput;

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
