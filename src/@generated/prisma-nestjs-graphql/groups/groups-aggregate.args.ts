import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereInput } from './groups-where.input';
import { GroupsOrderByWithRelationInput } from './groups-order-by-with-relation.input';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';
import { Int } from '@nestjs/graphql';
import { GroupsCountAggregateInput } from './groups-count-aggregate.input';
import { GroupsAvgAggregateInput } from './groups-avg-aggregate.input';
import { GroupsSumAggregateInput } from './groups-sum-aggregate.input';
import { GroupsMinAggregateInput } from './groups-min-aggregate.input';
import { GroupsMaxAggregateInput } from './groups-max-aggregate.input';

@ArgsType()
export class GroupsAggregateArgs {

    @Field(() => GroupsWhereInput, {nullable:true})
    where?: GroupsWhereInput;

    @Field(() => [GroupsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<GroupsOrderByWithRelationInput>;

    @Field(() => GroupsWhereUniqueInput, {nullable:true})
    cursor?: GroupsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => GroupsCountAggregateInput, {nullable:true})
    _count?: GroupsCountAggregateInput;

    @Field(() => GroupsAvgAggregateInput, {nullable:true})
    _avg?: GroupsAvgAggregateInput;

    @Field(() => GroupsSumAggregateInput, {nullable:true})
    _sum?: GroupsSumAggregateInput;

    @Field(() => GroupsMinAggregateInput, {nullable:true})
    _min?: GroupsMinAggregateInput;

    @Field(() => GroupsMaxAggregateInput, {nullable:true})
    _max?: GroupsMaxAggregateInput;
}
