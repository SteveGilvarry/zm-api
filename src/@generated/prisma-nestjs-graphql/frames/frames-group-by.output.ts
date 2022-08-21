import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Frames_Type } from '../prisma/frames-type.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { FramesCountAggregate } from './frames-count-aggregate.output';
import { FramesAvgAggregate } from './frames-avg-aggregate.output';
import { FramesSumAggregate } from './frames-sum-aggregate.output';
import { FramesMinAggregate } from './frames-min-aggregate.output';
import { FramesMaxAggregate } from './frames-max-aggregate.output';

@ObjectType()
export class FramesGroupBy {

    @Field(() => String, {nullable:false})
    Id!: bigint | number;

    @Field(() => String, {nullable:false})
    EventId!: bigint | number;

    @Field(() => Int, {nullable:false})
    FrameId!: number;

    @Field(() => Frames_Type, {nullable:false})
    Type!: keyof typeof Frames_Type;

    @Field(() => Date, {nullable:false})
    TimeStamp!: Date | string;

    @Field(() => GraphQLDecimal, {nullable:false})
    Delta!: Decimal;

    @Field(() => Int, {nullable:false})
    Score!: number;

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
