import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Frames_Type } from '../prisma/frames-type.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class FramesMaxAggregate {

    @Field(() => String, {nullable:true})
    Id?: bigint | number;

    @Field(() => String, {nullable:true})
    EventId?: bigint | number;

    @Field(() => Int, {nullable:true})
    FrameId?: number;

    @Field(() => Frames_Type, {nullable:true})
    Type?: keyof typeof Frames_Type;

    @Field(() => Date, {nullable:true})
    TimeStamp?: Date | string;

    @Field(() => GraphQLDecimal, {nullable:true})
    Delta?: Decimal;

    @Field(() => Int, {nullable:true})
    Score?: number;
}
