import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class FramesAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    EventId?: number;

    @Field(() => Float, {nullable:true})
    FrameId?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    Delta?: Decimal;

    @Field(() => Float, {nullable:true})
    Score?: number;
}
