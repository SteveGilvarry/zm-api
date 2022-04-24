import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { FramesCountAggregate } from './frames-count-aggregate.output';
import { FramesAvgAggregate } from './frames-avg-aggregate.output';
import { FramesSumAggregate } from './frames-sum-aggregate.output';
import { FramesMinAggregate } from './frames-min-aggregate.output';
import { FramesMaxAggregate } from './frames-max-aggregate.output';

@ObjectType()
export class AggregateFrames {

    @Field(() => FramesCountAggregate, {nullable:true})
    _count?: FramesCountAggregate;

    @Field(() => FramesAvgAggregate, {nullable:true})
    _avg?: FramesAvgAggregate;

    @Field(() => FramesSumAggregate, {nullable:true})
    _sum?: FramesSumAggregate;

    @Field(() => FramesMinAggregate, {nullable:true})
    _min?: FramesMinAggregate;

    @Field(() => FramesMaxAggregate, {nullable:true})
    _max?: FramesMaxAggregate;
}
