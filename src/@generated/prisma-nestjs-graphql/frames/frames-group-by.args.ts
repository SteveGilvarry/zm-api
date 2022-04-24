import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereInput } from './frames-where.input';
import { FramesOrderByWithAggregationInput } from './frames-order-by-with-aggregation.input';
import { FramesScalarFieldEnum } from './frames-scalar-field.enum';
import { FramesScalarWhereWithAggregatesInput } from './frames-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { FramesCountAggregateInput } from './frames-count-aggregate.input';
import { FramesAvgAggregateInput } from './frames-avg-aggregate.input';
import { FramesSumAggregateInput } from './frames-sum-aggregate.input';
import { FramesMinAggregateInput } from './frames-min-aggregate.input';
import { FramesMaxAggregateInput } from './frames-max-aggregate.input';

@ArgsType()
export class FramesGroupByArgs {

    @Field(() => FramesWhereInput, {nullable:true})
    where?: FramesWhereInput;

    @Field(() => [FramesOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<FramesOrderByWithAggregationInput>;

    @Field(() => [FramesScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof FramesScalarFieldEnum>;

    @Field(() => FramesScalarWhereWithAggregatesInput, {nullable:true})
    having?: FramesScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => FramesCountAggregateInput, {nullable:true})
    _count?: FramesCountAggregateInput;

    @Field(() => FramesAvgAggregateInput, {nullable:true})
    _avg?: FramesAvgAggregateInput;

    @Field(() => FramesSumAggregateInput, {nullable:true})
    _sum?: FramesSumAggregateInput;

    @Field(() => FramesMinAggregateInput, {nullable:true})
    _min?: FramesMinAggregateInput;

    @Field(() => FramesMaxAggregateInput, {nullable:true})
    _max?: FramesMaxAggregateInput;
}
