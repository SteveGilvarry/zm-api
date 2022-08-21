import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { MonitorPresets_Type } from '../prisma/monitor-presets-type.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { transformToDecimal } from 'prisma-graphql-type-decimal';
import { Transform } from 'class-transformer';
import { Type } from 'class-transformer';

@InputType()
export class MonitorPresetsCreateManyInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => MonitorPresets_Type, {nullable:true})
    Type?: keyof typeof MonitorPresets_Type;

    @Field(() => String, {nullable:true})
    Device?: string;

    @Field(() => String, {nullable:true})
    Channel?: string;

    @Field(() => Int, {nullable:true})
    Format?: number;

    @Field(() => String, {nullable:true})
    Protocol?: string;

    @Field(() => String, {nullable:true})
    Method?: string;

    @Field(() => String, {nullable:true})
    Host?: string;

    @Field(() => String, {nullable:true})
    Port?: string;

    @Field(() => String, {nullable:true})
    Path?: string;

    @Field(() => String, {nullable:true})
    SubPath?: string;

    @Field(() => Int, {nullable:true})
    Width?: number;

    @Field(() => Int, {nullable:true})
    Height?: number;

    @Field(() => Int, {nullable:true})
    Palette?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    @Type(() => Object)
    @Transform(transformToDecimal)
    MaxFPS?: Decimal;

    @Field(() => Int, {nullable:true})
    Controllable?: number;

    @Field(() => String, {nullable:true})
    ControlId?: string;

    @Field(() => String, {nullable:true})
    ControlDevice?: string;

    @Field(() => String, {nullable:true})
    ControlAddress?: string;

    @Field(() => Int, {nullable:true})
    DefaultRate?: number;

    @Field(() => Int, {nullable:true})
    DefaultScale?: number;
}
