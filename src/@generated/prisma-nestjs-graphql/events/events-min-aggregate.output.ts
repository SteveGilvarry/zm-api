import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Events_Orientation } from './events-orientation.enum';
import { Events_Scheme } from '../prisma/events-scheme.enum';

@ObjectType()
export class EventsMinAggregate {

    @Field(() => String, {nullable:true})
    Id?: bigint | number;

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => Int, {nullable:true})
    StorageId?: number;

    @Field(() => Int, {nullable:true})
    SecondaryStorageId?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => String, {nullable:true})
    Cause?: string;

    @Field(() => Date, {nullable:true})
    StartDateTime?: Date | string;

    @Field(() => Date, {nullable:true})
    EndDateTime?: Date | string;

    @Field(() => Int, {nullable:true})
    Width?: number;

    @Field(() => Int, {nullable:true})
    Height?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    Length?: any;

    @Field(() => Int, {nullable:true})
    Frames?: number;

    @Field(() => Int, {nullable:true})
    AlarmFrames?: number;

    @Field(() => String, {nullable:true})
    DefaultVideo?: string;

    @Field(() => Int, {nullable:true})
    SaveJPEGs?: number;

    @Field(() => Int, {nullable:true})
    TotScore?: number;

    @Field(() => Int, {nullable:true})
    AvgScore?: number;

    @Field(() => Int, {nullable:true})
    MaxScore?: number;

    @Field(() => Int, {nullable:true})
    Archived?: number;

    @Field(() => Int, {nullable:true})
    Videoed?: number;

    @Field(() => Int, {nullable:true})
    Uploaded?: number;

    @Field(() => Int, {nullable:true})
    Emailed?: number;

    @Field(() => Int, {nullable:true})
    Messaged?: number;

    @Field(() => Int, {nullable:true})
    Executed?: number;

    @Field(() => String, {nullable:true})
    Notes?: string;

    @Field(() => Int, {nullable:true})
    StateId?: number;

    @Field(() => Events_Orientation, {nullable:true})
    Orientation?: keyof typeof Events_Orientation;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;

    @Field(() => Events_Scheme, {nullable:true})
    Scheme?: keyof typeof Events_Scheme;

    @Field(() => Boolean, {nullable:true})
    Locked?: boolean;
}
