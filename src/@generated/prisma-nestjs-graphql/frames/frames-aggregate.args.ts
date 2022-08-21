import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereInput } from './frames-where.input';
import { Type } from 'class-transformer';
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
    @Type(() => FramesWhereInput)
    where?: FramesWhereInput;

    @Field(() => [FramesOrderByWithRelationInput], {nullable:true})
    @Type(() => FramesOrderByWithRelationInput)
    orderBy?: Array<FramesOrderByWithRelationInput>;

    @Field(() => FramesWhereUniqueInput, {nullable:true})
    @Type(() => FramesWhereUniqueInput)
    cursor?: FramesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => FramesCountAggregateInput, {nullable:true})
    @Type(() => FramesCountAggregateInput)
    _count?: FramesCountAggregateInput;

    @Field(() => FramesAvgAggregateInput, {nullable:true})
    @Type(() => FramesAvgAggregateInput)
    _avg?: FramesAvgAggregateInput;

    @Field(() => FramesSumAggregateInput, {nullable:true})
    @Type(() => FramesSumAggregateInput)
    _sum?: FramesSumAggregateInput;

    @Field(() => FramesMinAggregateInput, {nullable:true})
    @Type(() => FramesMinAggregateInput)
    _min?: FramesMinAggregateInput;

    @Field(() => FramesMaxAggregateInput, {nullable:true})
    @Type(() => FramesMaxAggregateInput)
    _max?: FramesMaxAggregateInput;
}
