import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { BigIntFilter } from '../prisma/big-int-filter.input';
import { IntFilter } from '../prisma/int-filter.input';
import { EnumFrames_TypeFilter } from '../prisma/enum-frames-type-filter.input';
import { DateTimeFilter } from '../prisma/date-time-filter.input';
import { DecimalFilter } from '../prisma/decimal-filter.input';

@InputType()
export class FramesWhereInput {

    @Field(() => [FramesWhereInput], {nullable:true})
    AND?: Array<FramesWhereInput>;

    @Field(() => [FramesWhereInput], {nullable:true})
    OR?: Array<FramesWhereInput>;

    @Field(() => [FramesWhereInput], {nullable:true})
    NOT?: Array<FramesWhereInput>;

    @Field(() => BigIntFilter, {nullable:true})
    Id?: BigIntFilter;

    @Field(() => BigIntFilter, {nullable:true})
    EventId?: BigIntFilter;

    @Field(() => IntFilter, {nullable:true})
    FrameId?: IntFilter;

    @Field(() => EnumFrames_TypeFilter, {nullable:true})
    Type?: EnumFrames_TypeFilter;

    @Field(() => DateTimeFilter, {nullable:true})
    TimeStamp?: DateTimeFilter;

    @Field(() => DecimalFilter, {nullable:true})
    Delta?: DecimalFilter;

    @Field(() => IntFilter, {nullable:true})
    Score?: IntFilter;
}
