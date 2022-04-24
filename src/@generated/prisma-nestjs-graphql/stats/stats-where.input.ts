import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { BigIntFilter } from '../prisma/big-int-filter.input';

@InputType()
export class StatsWhereInput {

    @Field(() => [StatsWhereInput], {nullable:true})
    AND?: Array<StatsWhereInput>;

    @Field(() => [StatsWhereInput], {nullable:true})
    OR?: Array<StatsWhereInput>;

    @Field(() => [StatsWhereInput], {nullable:true})
    NOT?: Array<StatsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    ZoneId?: IntFilter;

    @Field(() => BigIntFilter, {nullable:true})
    EventId?: BigIntFilter;

    @Field(() => IntFilter, {nullable:true})
    FrameId?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    PixelDiff?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AlarmPixels?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    FilterPixels?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    BlobPixels?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Blobs?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MinBlobSize?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MaxBlobSize?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MinX?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MaxX?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MinY?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MaxY?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Score?: IntFilter;
}
