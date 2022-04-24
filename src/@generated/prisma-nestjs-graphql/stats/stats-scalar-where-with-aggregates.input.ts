import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { BigIntWithAggregatesFilter } from '../prisma/big-int-with-aggregates-filter.input';

@InputType()
export class StatsScalarWhereWithAggregatesInput {

    @Field(() => [StatsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<StatsScalarWhereWithAggregatesInput>;

    @Field(() => [StatsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<StatsScalarWhereWithAggregatesInput>;

    @Field(() => [StatsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<StatsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    ZoneId?: IntWithAggregatesFilter;

    @Field(() => BigIntWithAggregatesFilter, {nullable:true})
    EventId?: BigIntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    FrameId?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    PixelDiff?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    AlarmPixels?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    FilterPixels?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    BlobPixels?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Blobs?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MinBlobSize?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MaxBlobSize?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MinX?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MaxX?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MinY?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MaxY?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Score?: IntWithAggregatesFilter;
}
