import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { StringNullableWithAggregatesFilter } from '../prisma/string-nullable-with-aggregates-filter.input';
import { EnumUsers_StreamWithAggregatesFilter } from '../prisma/enum-users-stream-with-aggregates-filter.input';
import { EnumUsers_EventsWithAggregatesFilter } from '../prisma/enum-users-events-with-aggregates-filter.input';
import { EnumUsers_ControlWithAggregatesFilter } from '../prisma/enum-users-control-with-aggregates-filter.input';
import { EnumUsers_MonitorsWithAggregatesFilter } from '../prisma/enum-users-monitors-with-aggregates-filter.input';
import { EnumUsers_GroupsWithAggregatesFilter } from '../prisma/enum-users-groups-with-aggregates-filter.input';
import { EnumUsers_DevicesWithAggregatesFilter } from '../prisma/enum-users-devices-with-aggregates-filter.input';
import { EnumUsers_SnapshotsWithAggregatesFilter } from '../prisma/enum-users-snapshots-with-aggregates-filter.input';
import { EnumUsers_SystemWithAggregatesFilter } from '../prisma/enum-users-system-with-aggregates-filter.input';
import { BigIntWithAggregatesFilter } from '../prisma/big-int-with-aggregates-filter.input';

@InputType()
export class UsersScalarWhereWithAggregatesInput {

    @Field(() => [UsersScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<UsersScalarWhereWithAggregatesInput>;

    @Field(() => [UsersScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<UsersScalarWhereWithAggregatesInput>;

    @Field(() => [UsersScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<UsersScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Username?: StringWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Password?: StringWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    Language?: StringNullableWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Enabled?: IntWithAggregatesFilter;

    @Field(() => EnumUsers_StreamWithAggregatesFilter, {nullable:true})
    Stream?: EnumUsers_StreamWithAggregatesFilter;

    @Field(() => EnumUsers_EventsWithAggregatesFilter, {nullable:true})
    Events?: EnumUsers_EventsWithAggregatesFilter;

    @Field(() => EnumUsers_ControlWithAggregatesFilter, {nullable:true})
    Control?: EnumUsers_ControlWithAggregatesFilter;

    @Field(() => EnumUsers_MonitorsWithAggregatesFilter, {nullable:true})
    Monitors?: EnumUsers_MonitorsWithAggregatesFilter;

    @Field(() => EnumUsers_GroupsWithAggregatesFilter, {nullable:true})
    Groups?: EnumUsers_GroupsWithAggregatesFilter;

    @Field(() => EnumUsers_DevicesWithAggregatesFilter, {nullable:true})
    Devices?: EnumUsers_DevicesWithAggregatesFilter;

    @Field(() => EnumUsers_SnapshotsWithAggregatesFilter, {nullable:true})
    Snapshots?: EnumUsers_SnapshotsWithAggregatesFilter;

    @Field(() => EnumUsers_SystemWithAggregatesFilter, {nullable:true})
    System?: EnumUsers_SystemWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    MaxBandwidth?: StringNullableWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    MonitorIds?: StringNullableWithAggregatesFilter;

    @Field(() => BigIntWithAggregatesFilter, {nullable:true})
    TokenMinExpiry?: BigIntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    APIEnabled?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    HomeView?: StringWithAggregatesFilter;
}
