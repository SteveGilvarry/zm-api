import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class EventsAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    MonitorId?: number;

    @Field(() => Float, {nullable:true})
    StorageId?: number;

    @Field(() => Float, {nullable:true})
    SecondaryStorageId?: number;

    @Field(() => Float, {nullable:true})
    Width?: number;

    @Field(() => Float, {nullable:true})
    Height?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    Length?: any;

    @Field(() => Float, {nullable:true})
    Frames?: number;

    @Field(() => Float, {nullable:true})
    AlarmFrames?: number;

    @Field(() => Float, {nullable:true})
    SaveJPEGs?: number;

    @Field(() => Float, {nullable:true})
    TotScore?: number;

    @Field(() => Float, {nullable:true})
    AvgScore?: number;

    @Field(() => Float, {nullable:true})
    MaxScore?: number;

    @Field(() => Float, {nullable:true})
    Archived?: number;

    @Field(() => Float, {nullable:true})
    Videoed?: number;

    @Field(() => Float, {nullable:true})
    Uploaded?: number;

    @Field(() => Float, {nullable:true})
    Emailed?: number;

    @Field(() => Float, {nullable:true})
    Messaged?: number;

    @Field(() => Float, {nullable:true})
    Executed?: number;

    @Field(() => Float, {nullable:true})
    StateId?: number;

    @Field(() => Float, {nullable:true})
    DiskSpace?: number;
}
