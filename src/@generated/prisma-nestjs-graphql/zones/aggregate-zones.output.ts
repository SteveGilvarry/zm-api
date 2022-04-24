import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ZonesCountAggregate } from './zones-count-aggregate.output';
import { ZonesAvgAggregate } from './zones-avg-aggregate.output';
import { ZonesSumAggregate } from './zones-sum-aggregate.output';
import { ZonesMinAggregate } from './zones-min-aggregate.output';
import { ZonesMaxAggregate } from './zones-max-aggregate.output';

@ObjectType()
export class AggregateZones {

    @Field(() => ZonesCountAggregate, {nullable:true})
    _count?: ZonesCountAggregate;

    @Field(() => ZonesAvgAggregate, {nullable:true})
    _avg?: ZonesAvgAggregate;

    @Field(() => ZonesSumAggregate, {nullable:true})
    _sum?: ZonesSumAggregate;

    @Field(() => ZonesMinAggregate, {nullable:true})
    _min?: ZonesMinAggregate;

    @Field(() => ZonesMaxAggregate, {nullable:true})
    _max?: ZonesMaxAggregate;
}
