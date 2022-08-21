import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class ServersSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    Port?: number;

    @Field(() => Int, {nullable:true})
    State_Id?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    CpuLoad?: Decimal;

    @Field(() => String, {nullable:true})
    TotalMem?: bigint | number;

    @Field(() => String, {nullable:true})
    FreeMem?: bigint | number;

    @Field(() => String, {nullable:true})
    TotalSwap?: bigint | number;

    @Field(() => String, {nullable:true})
    FreeSwap?: bigint | number;
}
