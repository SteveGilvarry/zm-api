import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class FramesCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    EventId!: number;

    @Field(() => Int, {nullable:false})
    FrameId!: number;

    @Field(() => Int, {nullable:false})
    Type!: number;

    @Field(() => Int, {nullable:false})
    TimeStamp!: number;

    @Field(() => Int, {nullable:false})
    Delta!: number;

    @Field(() => Int, {nullable:false})
    Score!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
