import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class UsersCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Username!: number;

    @Field(() => Int, {nullable:false})
    Password!: number;

    @Field(() => Int, {nullable:false})
    Language!: number;

    @Field(() => Int, {nullable:false})
    Enabled!: number;

    @Field(() => Int, {nullable:false})
    Stream!: number;

    @Field(() => Int, {nullable:false})
    Events!: number;

    @Field(() => Int, {nullable:false})
    Control!: number;

    @Field(() => Int, {nullable:false})
    Monitors!: number;

    @Field(() => Int, {nullable:false})
    Groups!: number;

    @Field(() => Int, {nullable:false})
    Devices!: number;

    @Field(() => Int, {nullable:false})
    Snapshots!: number;

    @Field(() => Int, {nullable:false})
    System!: number;

    @Field(() => Int, {nullable:false})
    MaxBandwidth!: number;

    @Field(() => Int, {nullable:false})
    MonitorIds!: number;

    @Field(() => Int, {nullable:false})
    TokenMinExpiry!: number;

    @Field(() => Int, {nullable:false})
    APIEnabled!: number;

    @Field(() => Int, {nullable:false})
    HomeView!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
