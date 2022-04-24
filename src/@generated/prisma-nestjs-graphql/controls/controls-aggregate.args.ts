import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereInput } from './controls-where.input';
import { ControlsOrderByWithRelationInput } from './controls-order-by-with-relation.input';
import { ControlsWhereUniqueInput } from './controls-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ControlsCountAggregateInput } from './controls-count-aggregate.input';
import { ControlsAvgAggregateInput } from './controls-avg-aggregate.input';
import { ControlsSumAggregateInput } from './controls-sum-aggregate.input';
import { ControlsMinAggregateInput } from './controls-min-aggregate.input';
import { ControlsMaxAggregateInput } from './controls-max-aggregate.input';

@ArgsType()
export class ControlsAggregateArgs {

    @Field(() => ControlsWhereInput, {nullable:true})
    where?: ControlsWhereInput;

    @Field(() => [ControlsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ControlsOrderByWithRelationInput>;

    @Field(() => ControlsWhereUniqueInput, {nullable:true})
    cursor?: ControlsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ControlsCountAggregateInput, {nullable:true})
    _count?: ControlsCountAggregateInput;

    @Field(() => ControlsAvgAggregateInput, {nullable:true})
    _avg?: ControlsAvgAggregateInput;

    @Field(() => ControlsSumAggregateInput, {nullable:true})
    _sum?: ControlsSumAggregateInput;

    @Field(() => ControlsMinAggregateInput, {nullable:true})
    _min?: ControlsMinAggregateInput;

    @Field(() => ControlsMaxAggregateInput, {nullable:true})
    _max?: ControlsMaxAggregateInput;
}
