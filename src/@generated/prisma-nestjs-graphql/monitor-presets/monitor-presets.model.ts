import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { MonitorPresets_Type } from '../prisma/monitor-presets-type.enum';
import { Int } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class MonitorPresets {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false,defaultValue:''})
    Name!: string;

    @Field(() => MonitorPresets_Type, {nullable:false,defaultValue:'Local'})
    Type!: keyof typeof MonitorPresets_Type;

    @Field(() => String, {nullable:true})
    Device!: string | null;

    @Field(() => String, {nullable:true})
    Channel!: string | null;

    @Field(() => Int, {nullable:true})
    Format!: number | null;

    @Field(() => String, {nullable:true})
    Protocol!: string | null;

    @Field(() => String, {nullable:true})
    Method!: string | null;

    @Field(() => String, {nullable:true})
    Host!: string | null;

    @Field(() => String, {nullable:true})
    Port!: string | null;

    @Field(() => String, {nullable:true})
    Path!: string | null;

    @Field(() => String, {nullable:true})
    SubPath!: string | null;

    @Field(() => Int, {nullable:true})
    Width!: number | null;

    @Field(() => Int, {nullable:true})
    Height!: number | null;

    @Field(() => Int, {nullable:true})
    Palette!: number | null;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS!: any | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Controllable!: number;

    @Field(() => String, {nullable:true})
    ControlId!: string | null;

    @Field(() => String, {nullable:true})
    ControlDevice!: string | null;

    @Field(() => String, {nullable:true})
    ControlAddress!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:100})
    DefaultRate!: number;

    @Field(() => Int, {nullable:false,defaultValue:100})
    DefaultScale!: number;
}
