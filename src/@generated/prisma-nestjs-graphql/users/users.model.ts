import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
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
export class Users {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false,defaultValue:''})
    Username!: string;

    @Field(() => String, {nullable:false,defaultValue:''})
    Password!: string;

    @Field(() => String, {nullable:true})
    Language!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:1})
    Enabled!: number;

    @Field(() => Users_Stream, {nullable:false,defaultValue:'None'})
    Stream!: keyof typeof Users_Stream;

    @Field(() => Users_Events, {nullable:false,defaultValue:'None'})
    Events!: keyof typeof Users_Events;

    @Field(() => Users_Control, {nullable:false,defaultValue:'None'})
    Control!: keyof typeof Users_Control;

    @Field(() => Users_Monitors, {nullable:false,defaultValue:'None'})
    Monitors!: keyof typeof Users_Monitors;

    @Field(() => Users_Groups, {nullable:false,defaultValue:'None'})
    Groups!: keyof typeof Users_Groups;

    @Field(() => Users_Devices, {nullable:false,defaultValue:'None'})
    Devices!: keyof typeof Users_Devices;

    @Field(() => Users_Snapshots, {nullable:false,defaultValue:'None'})
    Snapshots!: keyof typeof Users_Snapshots;

    @Field(() => Users_System, {nullable:false,defaultValue:'None'})
    System!: keyof typeof Users_System;

    @Field(() => String, {nullable:true})
    MaxBandwidth!: string | null;

    @Field(() => String, {nullable:true})
    MonitorIds!: string | null;

    @Field(() => String, {nullable:false,defaultValue:'0'})
    TokenMinExpiry!: bigint;

    @Field(() => Int, {nullable:false,defaultValue:1})
    APIEnabled!: number;

    @Field(() => String, {nullable:false,defaultValue:''})
    HomeView!: string;
}
