import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class FramesSumAggregate {

    @Field(() => String, {nullable:true})
    Id?: bigint | number;

    @Field(() => String, {nullable:true})
    EventId?: bigint | number;

    @Field(() => Int, {nullable:true})
    FrameId?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    Delta?: any;

    @Field(() => Int, {nullable:true})
    Score?: number;
}
