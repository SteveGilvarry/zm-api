import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Type } from 'class-transformer';
import { BigIntWithAggregatesFilter } from '../prisma/big-int-with-aggregates-filter.input';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { EnumFrames_TypeWithAggregatesFilter } from '../prisma/enum-frames-type-with-aggregates-filter.input';
import { DateTimeWithAggregatesFilter } from '../prisma/date-time-with-aggregates-filter.input';
import { DecimalWithAggregatesFilter } from '../prisma/decimal-with-aggregates-filter.input';

@InputType()
export class FramesScalarWhereWithAggregatesInput {

    @Field(() => [FramesScalarWhereWithAggregatesInput], {nullable:true})
    @Type(() => FramesScalarWhereWithAggregatesInput)
    AND?: Array<FramesScalarWhereWithAggregatesInput>;

    @Field(() => [FramesScalarWhereWithAggregatesInput], {nullable:true})
    @Type(() => FramesScalarWhereWithAggregatesInput)
    OR?: Array<FramesScalarWhereWithAggregatesInput>;

    @Field(() => [FramesScalarWhereWithAggregatesInput], {nullable:true})
    @Type(() => FramesScalarWhereWithAggregatesInput)
    NOT?: Array<FramesScalarWhereWithAggregatesInput>;

    @Field(() => BigIntWithAggregatesFilter, {nullable:true})
    Id?: BigIntWithAggregatesFilter;

    @Field(() => BigIntWithAggregatesFilter, {nullable:true})
    EventId?: BigIntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    FrameId?: IntWithAggregatesFilter;

    @Field(() => EnumFrames_TypeWithAggregatesFilter, {nullable:true})
    Type?: EnumFrames_TypeWithAggregatesFilter;

    @Field(() => DateTimeWithAggregatesFilter, {nullable:true})
    TimeStamp?: DateTimeWithAggregatesFilter;

    @Field(() => DecimalWithAggregatesFilter, {nullable:true})
    @Type(() => DecimalWithAggregatesFilter)
    Delta?: DecimalWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Score?: IntWithAggregatesFilter;
}
