import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { BigIntFilter } from '../prisma/big-int-filter.input';
import { IntFilter } from '../prisma/int-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { DateTimeNullableFilter } from '../prisma/date-time-nullable-filter.input';
import { DecimalFilter } from '../prisma/decimal-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';
import { EnumEvents_OrientationFilter } from '../prisma/enum-events-orientation-filter.input';
import { BigIntNullableFilter } from '../prisma/big-int-nullable-filter.input';
import { EnumEvents_SchemeFilter } from '../prisma/enum-events-scheme-filter.input';
import { BoolFilter } from '../prisma/bool-filter.input';

@InputType()
export class EventsWhereInput {

    @Field(() => [EventsWhereInput], {nullable:true})
    AND?: Array<EventsWhereInput>;

    @Field(() => [EventsWhereInput], {nullable:true})
    OR?: Array<EventsWhereInput>;

    @Field(() => [EventsWhereInput], {nullable:true})
    NOT?: Array<EventsWhereInput>;

    @Field(() => BigIntFilter, {nullable:true})
    Id?: BigIntFilter;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    StorageId?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    SecondaryStorageId?: IntNullableFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    Cause?: StringFilter;

    @Field(() => DateTimeNullableFilter, {nullable:true})
    StartDateTime?: DateTimeNullableFilter;

    @Field(() => DateTimeNullableFilter, {nullable:true})
    EndDateTime?: DateTimeNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    Width?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Height?: IntFilter;

    @Field(() => DecimalFilter, {nullable:true})
    Length?: DecimalFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    Frames?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    AlarmFrames?: IntNullableFilter;

    @Field(() => StringFilter, {nullable:true})
    DefaultVideo?: StringFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    SaveJPEGs?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    TotScore?: IntFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    AvgScore?: IntNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    MaxScore?: IntNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    Archived?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Videoed?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Uploaded?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Emailed?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Messaged?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Executed?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Notes?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    StateId?: IntFilter;

    @Field(() => EnumEvents_OrientationFilter, {nullable:true})
    Orientation?: EnumEvents_OrientationFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    DiskSpace?: BigIntNullableFilter;

    @Field(() => EnumEvents_SchemeFilter, {nullable:true})
    Scheme?: EnumEvents_SchemeFilter;

    @Field(() => BoolFilter, {nullable:true})
    Locked?: BoolFilter;
}
