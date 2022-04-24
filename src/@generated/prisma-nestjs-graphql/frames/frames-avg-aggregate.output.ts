import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';
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
    Delta?: any;

    @Field(() => Float, {nullable:true})
    Score?: number;
}
