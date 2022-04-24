import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Users_Stream } from '../prisma/users-stream.enum';
import { Users_Events } from '../prisma/users-events.enum';
import { Users_Control } from '../prisma/users-control.enum';
import { Users_Monitors } from '../prisma/users-monitors.enum';
import { Users_Groups } from '../prisma/users-groups.enum';
import { Users_Devices } from '../prisma/users-devices.enum';
import { Users_Snapshots } from '../prisma/users-snapshots.enum';
import { Users_System } from '../prisma/users-system.enum';
import { UsersCountAggregate } from './users-count-aggregate.output';
import { UsersAvgAggregate } from './users-avg-aggregate.output';
import { UsersSumAggregate } from './users-sum-aggregate.output';
import { UsersMinAggregate } from './users-min-aggregate.output';
import { UsersMaxAggregate } from './users-max-aggregate.output';

@ObjectType()
export class UsersGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Username!: string;

    @Field(() => String, {nullable:false})
    Password!: string;

    @Field(() => String, {nullable:true})
    Language?: string;

    @Field(() => Int, {nullable:false})
    Enabled!: number;

    @Field(() => Users_Stream, {nullable:false})
    Stream!: keyof typeof Users_Stream;

    @Field(() => Users_Events, {nullable:false})
    Events!: keyof typeof Users_Events;

    @Field(() => Users_Control, {nullable:false})
    Control!: keyof typeof Users_Control;

    @Field(() => Users_Monitors, {nullable:false})
    Monitors!: keyof typeof Users_Monitors;

    @Field(() => Users_Groups, {nullable:false})
    Groups!: keyof typeof Users_Groups;

    @Field(() => Users_Devices, {nullable:false})
    Devices!: keyof typeof Users_Devices;

    @Field(() => Users_Snapshots, {nullable:false})
    Snapshots!: keyof typeof Users_Snapshots;

    @Field(() => Users_System, {nullable:false})
    System!: keyof typeof Users_System;

    @Field(() => String, {nullable:true})
    MaxBandwidth?: string;

    @Field(() => String, {nullable:true})
    MonitorIds?: string;

    @Field(() => String, {nullable:false})
    TokenMinExpiry!: bigint | number;

    @Field(() => Int, {nullable:false})
    APIEnabled!: number;

    @Field(() => String, {nullable:false})
    HomeView!: string;

    @Field(() => UsersCountAggregate, {nullable:true})
    _count?: UsersCountAggregate;

    @Field(() => UsersAvgAggregate, {nullable:true})
    _avg?: UsersAvgAggregate;

    @Field(() => UsersSumAggregate, {nullable:true})
    _sum?: UsersSumAggregate;

    @Field(() => UsersMinAggregate, {nullable:true})
    _min?: UsersMinAggregate;

    @Field(() => UsersMaxAggregate, {nullable:true})
    _max?: UsersMaxAggregate;
}
