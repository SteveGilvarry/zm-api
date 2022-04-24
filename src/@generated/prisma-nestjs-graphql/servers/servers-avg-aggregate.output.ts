import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class ServersAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    Port?: number;

    @Field(() => Float, {nullable:true})
    State_Id?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    CpuLoad?: any;

    @Field(() => Float, {nullable:true})
    TotalMem?: number;

    @Field(() => Float, {nullable:true})
    FreeMem?: number;

    @Field(() => Float, {nullable:true})
    TotalSwap?: number;

    @Field(() => Float, {nullable:true})
    FreeSwap?: number;
}
