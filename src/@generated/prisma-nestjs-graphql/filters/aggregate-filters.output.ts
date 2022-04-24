import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { FiltersCountAggregate } from './filters-count-aggregate.output';
import { FiltersAvgAggregate } from './filters-avg-aggregate.output';
import { FiltersSumAggregate } from './filters-sum-aggregate.output';
import { FiltersMinAggregate } from './filters-min-aggregate.output';
import { FiltersMaxAggregate } from './filters-max-aggregate.output';

@ObjectType()
export class AggregateFilters {

    @Field(() => FiltersCountAggregate, {nullable:true})
    _count?: FiltersCountAggregate;

    @Field(() => FiltersAvgAggregate, {nullable:true})
    _avg?: FiltersAvgAggregate;

    @Field(() => FiltersSumAggregate, {nullable:true})
    _sum?: FiltersSumAggregate;

    @Field(() => FiltersMinAggregate, {nullable:true})
    _min?: FiltersMinAggregate;

    @Field(() => FiltersMaxAggregate, {nullable:true})
    _max?: FiltersMaxAggregate;
}
