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

@ObjectType()
export class UsersMaxAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Username?: string;

    @Field(() => String, {nullable:true})
    Password?: string;

    @Field(() => String, {nullable:true})
    Language?: string;

    @Field(() => Int, {nullable:true})
    Enabled?: number;

    @Field(() => Users_Stream, {nullable:true})
    Stream?: keyof typeof Users_Stream;

    @Field(() => Users_Events, {nullable:true})
    Events?: keyof typeof Users_Events;

    @Field(() => Users_Control, {nullable:true})
    Control?: keyof typeof Users_Control;

    @Field(() => Users_Monitors, {nullable:true})
    Monitors?: keyof typeof Users_Monitors;

    @Field(() => Users_Groups, {nullable:true})
    Groups?: keyof typeof Users_Groups;

    @Field(() => Users_Devices, {nullable:true})
    Devices?: keyof typeof Users_Devices;

    @Field(() => Users_Snapshots, {nullable:true})
    Snapshots?: keyof typeof Users_Snapshots;

    @Field(() => Users_System, {nullable:true})
    System?: keyof typeof Users_System;

    @Field(() => String, {nullable:true})
    MaxBandwidth?: string;

    @Field(() => String, {nullable:true})
    MonitorIds?: string;

    @Field(() => String, {nullable:true})
    TokenMinExpiry?: bigint | number;

    @Field(() => Int, {nullable:true})
    APIEnabled?: number;

    @Field(() => String, {nullable:true})
    HomeView?: string;
}
