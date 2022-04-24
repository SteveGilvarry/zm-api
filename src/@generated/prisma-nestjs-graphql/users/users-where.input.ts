import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';
import { EnumUsers_StreamFilter } from '../prisma/enum-users-stream-filter.input';
import { EnumUsers_EventsFilter } from '../prisma/enum-users-events-filter.input';
import { EnumUsers_ControlFilter } from '../prisma/enum-users-control-filter.input';
import { EnumUsers_MonitorsFilter } from '../prisma/enum-users-monitors-filter.input';
import { EnumUsers_GroupsFilter } from '../prisma/enum-users-groups-filter.input';
import { EnumUsers_DevicesFilter } from '../prisma/enum-users-devices-filter.input';
import { EnumUsers_SnapshotsFilter } from '../prisma/enum-users-snapshots-filter.input';
import { EnumUsers_SystemFilter } from '../prisma/enum-users-system-filter.input';
import { BigIntFilter } from '../prisma/big-int-filter.input';

@InputType()
export class UsersWhereInput {

    @Field(() => [UsersWhereInput], {nullable:true})
    AND?: Array<UsersWhereInput>;

    @Field(() => [UsersWhereInput], {nullable:true})
    OR?: Array<UsersWhereInput>;

    @Field(() => [UsersWhereInput], {nullable:true})
    NOT?: Array<UsersWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Username?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    Password?: StringFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Language?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    Enabled?: IntFilter;

    @Field(() => EnumUsers_StreamFilter, {nullable:true})
    Stream?: EnumUsers_StreamFilter;

    @Field(() => EnumUsers_EventsFilter, {nullable:true})
    Events?: EnumUsers_EventsFilter;

    @Field(() => EnumUsers_ControlFilter, {nullable:true})
    Control?: EnumUsers_ControlFilter;

    @Field(() => EnumUsers_MonitorsFilter, {nullable:true})
    Monitors?: EnumUsers_MonitorsFilter;

    @Field(() => EnumUsers_GroupsFilter, {nullable:true})
    Groups?: EnumUsers_GroupsFilter;

    @Field(() => EnumUsers_DevicesFilter, {nullable:true})
    Devices?: EnumUsers_DevicesFilter;

    @Field(() => EnumUsers_SnapshotsFilter, {nullable:true})
    Snapshots?: EnumUsers_SnapshotsFilter;

    @Field(() => EnumUsers_SystemFilter, {nullable:true})
    System?: EnumUsers_SystemFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    MaxBandwidth?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    MonitorIds?: StringNullableFilter;

    @Field(() => BigIntFilter, {nullable:true})
    TokenMinExpiry?: BigIntFilter;

    @Field(() => IntFilter, {nullable:true})
    APIEnabled?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    HomeView?: StringFilter;
}
