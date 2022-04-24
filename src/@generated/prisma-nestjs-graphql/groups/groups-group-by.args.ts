import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereInput } from './groups-where.input';
import { GroupsOrderByWithAggregationInput } from './groups-order-by-with-aggregation.input';
import { GroupsScalarFieldEnum } from './groups-scalar-field.enum';
import { GroupsScalarWhereWithAggregatesInput } from './groups-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { GroupsCountAggregateInput } from './groups-count-aggregate.input';
import { GroupsAvgAggregateInput } from './groups-avg-aggregate.input';
import { GroupsSumAggregateInput } from './groups-sum-aggregate.input';
import { GroupsMinAggregateInput } from './groups-min-aggregate.input';
import { GroupsMaxAggregateInput } from './groups-max-aggregate.input';

@ArgsType()
export class GroupsGroupByArgs {

    @Field(() => GroupsWhereInput, {nullable:true})
    where?: GroupsWhereInput;

    @Field(() => [GroupsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<GroupsOrderByWithAggregationInput>;

    @Field(() => [GroupsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof GroupsScalarFieldEnum>;

    @Field(() => GroupsScalarWhereWithAggregatesInput, {nullable:true})
    having?: GroupsScalarWhereWithAggregatesInput;

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
