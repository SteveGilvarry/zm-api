import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereInput } from './frames-where.input';
import { FramesOrderByWithRelationInput } from './frames-order-by-with-relation.input';
import { FramesWhereUniqueInput } from './frames-where-unique.input';
import { Int } from '@nestjs/graphql';
import { FramesCountAggregateInput } from './frames-count-aggregate.input';
import { FramesAvgAggregateInput } from './frames-avg-aggregate.input';
import { FramesSumAggregateInput } from './frames-sum-aggregate.input';
import { FramesMinAggregateInput } from './frames-min-aggregate.input';
import { FramesMaxAggregateInput } from './frames-max-aggregate.input';

@ArgsType()
export class FramesAggregateArgs {

    @Field(() => FramesWhereInput, {nullable:true})
    where?: FramesWhereInput;

    @Field(() => [FramesOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<FramesOrderByWithRelationInput>;

    @Field(() => FramesWhereUniqueInput, {nullable:true})
    cursor?: FramesWhereUniqueInput;

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
