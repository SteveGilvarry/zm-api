import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { MontageLayoutsCountAggregate } from './montage-layouts-count-aggregate.output';
import { MontageLayoutsAvgAggregate } from './montage-layouts-avg-aggregate.output';
import { MontageLayoutsSumAggregate } from './montage-layouts-sum-aggregate.output';
import { MontageLayoutsMinAggregate } from './montage-layouts-min-aggregate.output';
import { MontageLayoutsMaxAggregate } from './montage-layouts-max-aggregate.output';

@ObjectType()
export class MontageLayoutsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => String, {nullable:true})
    Positions?: string;

    @Field(() => MontageLayoutsCountAggregate, {nullable:true})
    _count?: MontageLayoutsCountAggregate;

    @Field(() => MontageLayoutsAvgAggregate, {nullable:true})
    _avg?: MontageLayoutsAvgAggregate;

    @Field(() => MontageLayoutsSumAggregate, {nullable:true})
    _sum?: MontageLayoutsSumAggregate;

    @Field(() => MontageLayoutsMinAggregate, {nullable:true})
    _min?: MontageLayoutsMinAggregate;

    @Field(() => MontageLayoutsMaxAggregate, {nullable:true})
    _max?: MontageLayoutsMaxAggregate;
}
