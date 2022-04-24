import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { GroupsCountAggregate } from './groups-count-aggregate.output';
import { GroupsAvgAggregate } from './groups-avg-aggregate.output';
import { GroupsSumAggregate } from './groups-sum-aggregate.output';
import { GroupsMinAggregate } from './groups-min-aggregate.output';
import { GroupsMaxAggregate } from './groups-max-aggregate.output';

@ObjectType()
export class GroupsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Int, {nullable:true})
    ParentId?: number;

    @Field(() => GroupsCountAggregate, {nullable:true})
    _count?: GroupsCountAggregate;

    @Field(() => GroupsAvgAggregate, {nullable:true})
    _avg?: GroupsAvgAggregate;

    @Field(() => GroupsSumAggregate, {nullable:true})
    _sum?: GroupsSumAggregate;

    @Field(() => GroupsMinAggregate, {nullable:true})
    _min?: GroupsMinAggregate;

    @Field(() => GroupsMaxAggregate, {nullable:true})
    _max?: GroupsMaxAggregate;
}
