import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class MonitorPresetsSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    Format?: number;

    @Field(() => Int, {nullable:true})
    Width?: number;

    @Field(() => Int, {nullable:true})
    Height?: number;

    @Field(() => Int, {nullable:true})
    Palette?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS?: Decimal;

    @Field(() => Int, {nullable:true})
    Controllable?: number;

    @Field(() => Int, {nullable:true})
    DefaultRate?: number;

    @Field(() => Int, {nullable:true})
    DefaultScale?: number;
}
