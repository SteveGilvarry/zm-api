import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFieldUpdateOperationsInput } from '../prisma/int-field-update-operations.input';
import { StringFieldUpdateOperationsInput } from '../prisma/string-field-update-operations.input';
import { NullableStringFieldUpdateOperationsInput } from '../prisma/nullable-string-field-update-operations.input';
import { EnumUsers_StreamFieldUpdateOperationsInput } from '../prisma/enum-users-stream-field-update-operations.input';
import { EnumUsers_EventsFieldUpdateOperationsInput } from '../prisma/enum-users-events-field-update-operations.input';
import { EnumUsers_ControlFieldUpdateOperationsInput } from '../prisma/enum-users-control-field-update-operations.input';
import { EnumUsers_MonitorsFieldUpdateOperationsInput } from '../prisma/enum-users-monitors-field-update-operations.input';
import { EnumUsers_GroupsFieldUpdateOperationsInput } from '../prisma/enum-users-groups-field-update-operations.input';
import { EnumUsers_DevicesFieldUpdateOperationsInput } from '../prisma/enum-users-devices-field-update-operations.input';
import { EnumUsers_SnapshotsFieldUpdateOperationsInput } from '../prisma/enum-users-snapshots-field-update-operations.input';
import { EnumUsers_SystemFieldUpdateOperationsInput } from '../prisma/enum-users-system-field-update-operations.input';
import { BigIntFieldUpdateOperationsInput } from '../prisma/big-int-field-update-operations.input';

@InputType()
export class UsersUncheckedUpdateManyInput {

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    Id?: IntFieldUpdateOperationsInput;

    @Field(() => StringFieldUpdateOperationsInput, {nullable:true})
    Username?: StringFieldUpdateOperationsInput;

    @Field(() => StringFieldUpdateOperationsInput, {nullable:true})
    Password?: StringFieldUpdateOperationsInput;

    @Field(() => NullableStringFieldUpdateOperationsInput, {nullable:true})
    Language?: NullableStringFieldUpdateOperationsInput;

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    Enabled?: IntFieldUpdateOperationsInput;

    @Field(() => EnumUsers_StreamFieldUpdateOperationsInput, {nullable:true})
    Stream?: EnumUsers_StreamFieldUpdateOperationsInput;

    @Field(() => EnumUsers_EventsFieldUpdateOperationsInput, {nullable:true})
    Events?: EnumUsers_EventsFieldUpdateOperationsInput;

    @Field(() => EnumUsers_ControlFieldUpdateOperationsInput, {nullable:true})
    Control?: EnumUsers_ControlFieldUpdateOperationsInput;

    @Field(() => EnumUsers_MonitorsFieldUpdateOperationsInput, {nullable:true})
    Monitors?: EnumUsers_MonitorsFieldUpdateOperationsInput;

    @Field(() => EnumUsers_GroupsFieldUpdateOperationsInput, {nullable:true})
    Groups?: EnumUsers_GroupsFieldUpdateOperationsInput;

    @Field(() => EnumUsers_DevicesFieldUpdateOperationsInput, {nullable:true})
    Devices?: EnumUsers_DevicesFieldUpdateOperationsInput;

    @Field(() => EnumUsers_SnapshotsFieldUpdateOperationsInput, {nullable:true})
    Snapshots?: EnumUsers_SnapshotsFieldUpdateOperationsInput;

    @Field(() => EnumUsers_SystemFieldUpdateOperationsInput, {nullable:true})
    System?: EnumUsers_SystemFieldUpdateOperationsInput;

    @Field(() => NullableStringFieldUpdateOperationsInput, {nullable:true})
    MaxBandwidth?: NullableStringFieldUpdateOperationsInput;

    @Field(() => NullableStringFieldUpdateOperationsInput, {nullable:true})
    MonitorIds?: NullableStringFieldUpdateOperationsInput;

    @Field(() => BigIntFieldUpdateOperationsInput, {nullable:true})
    TokenMinExpiry?: BigIntFieldUpdateOperationsInput;

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    APIEnabled?: IntFieldUpdateOperationsInput;

    @Field(() => StringFieldUpdateOperationsInput, {nullable:true})
    HomeView?: StringFieldUpdateOperationsInput;
}
