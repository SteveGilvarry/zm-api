import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { ManufacturersCountAggregate } from './manufacturers-count-aggregate.output';
import { ManufacturersAvgAggregate } from './manufacturers-avg-aggregate.output';
import { ManufacturersSumAggregate } from './manufacturers-sum-aggregate.output';
import { ManufacturersMinAggregate } from './manufacturers-min-aggregate.output';
import { ManufacturersMaxAggregate } from './manufacturers-max-aggregate.output';

@ObjectType()
export class ManufacturersGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => ManufacturersCountAggregate, {nullable:true})
    _count?: ManufacturersCountAggregate;

    @Field(() => ManufacturersAvgAggregate, {nullable:true})
    _avg?: ManufacturersAvgAggregate;

    @Field(() => ManufacturersSumAggregate, {nullable:true})
    _sum?: ManufacturersSumAggregate;

    @Field(() => ManufacturersMinAggregate, {nullable:true})
    _min?: ManufacturersMinAggregate;

    @Field(() => ManufacturersMaxAggregate, {nullable:true})
    _max?: ManufacturersMaxAggregate;
}
