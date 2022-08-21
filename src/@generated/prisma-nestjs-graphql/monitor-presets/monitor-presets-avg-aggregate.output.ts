import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class MonitorPresetsAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    Format?: number;

    @Field(() => Float, {nullable:true})
    Width?: number;

    @Field(() => Float, {nullable:true})
    Height?: number;

    @Field(() => Float, {nullable:true})
    Palette?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS?: Decimal;

    @Field(() => Float, {nullable:true})
    Controllable?: number;

    @Field(() => Float, {nullable:true})
    DefaultRate?: number;

    @Field(() => Float, {nullable:true})
    DefaultScale?: number;
}
