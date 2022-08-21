import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Frames_Type } from '../prisma/frames-type.enum';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Decimal } from '@prisma/client/runtime';

@ObjectType()
export class Frames {

    @Field(() => ID, {nullable:false})
    Id!: bigint;

    @Field(() => String, {nullable:false})
    EventId!: bigint;

    @Field(() => Int, {nullable:false,defaultValue:0})
    FrameId!: number;

    @Field(() => Frames_Type, {nullable:false,defaultValue:'Normal'})
    Type!: keyof typeof Frames_Type;

    @Field(() => Date, {nullable:false})
    TimeStamp!: Date;

    @Field(() => GraphQLDecimal, {nullable:false,defaultValue:0})
    Delta!: Decimal;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Score!: number;
}
