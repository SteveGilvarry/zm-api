import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Decimal } from '@prisma/client/runtime';
import { Events_Orientation } from './events-orientation.enum';
import { Events_Scheme } from '../prisma/events-scheme.enum';

@ObjectType()
export class Events {

    @Field(() => ID, {nullable:false})
    Id!: bigint;

    @Field(() => Int, {nullable:false,defaultValue:0})
    MonitorId!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    StorageId!: number;

    @Field(() => Int, {nullable:true,defaultValue:0})
    SecondaryStorageId!: number | null;

    @Field(() => String, {nullable:false,defaultValue:''})
    Name!: string;

    @Field(() => String, {nullable:false,defaultValue:''})
    Cause!: string;

    @Field(() => Date, {nullable:true})
    StartDateTime!: Date | null;

    @Field(() => Date, {nullable:true})
    EndDateTime!: Date | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Width!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Height!: number;

    @Field(() => GraphQLDecimal, {nullable:false,defaultValue:0})
    Length!: Decimal;

    @Field(() => Int, {nullable:true})
    Frames!: number | null;

    @Field(() => Int, {nullable:true})
    AlarmFrames!: number | null;

    @Field(() => String, {nullable:false})
    DefaultVideo!: string;

    @Field(() => Int, {nullable:true})
    SaveJPEGs!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    TotScore!: number;

    @Field(() => Int, {nullable:true,defaultValue:0})
    AvgScore!: number | null;

    @Field(() => Int, {nullable:true,defaultValue:0})
    MaxScore!: number | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Archived!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Videoed!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Uploaded!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Emailed!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Messaged!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Executed!: number;

    @Field(() => String, {nullable:true})
    Notes!: string | null;

    @Field(() => Int, {nullable:false})
    StateId!: number;

    @Field(() => Events_Orientation, {nullable:false,defaultValue:'ROTATE_0'})
    Orientation!: keyof typeof Events_Orientation;

    @Field(() => String, {nullable:true})
    DiskSpace!: bigint | null;

    @Field(() => Events_Scheme, {nullable:false,defaultValue:'Deep'})
    Scheme!: keyof typeof Events_Scheme;

    @Field(() => Boolean, {nullable:false,defaultValue:false})
    Locked!: boolean;
}
