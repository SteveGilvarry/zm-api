import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class StatsMaxAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    ZoneId?: true;

    @Field(() => Boolean, {nullable:true})
    EventId?: true;

    @Field(() => Boolean, {nullable:true})
    FrameId?: true;

    @Field(() => Boolean, {nullable:true})
    PixelDiff?: true;

    @Field(() => Boolean, {nullable:true})
    AlarmPixels?: true;

    @Field(() => Boolean, {nullable:true})
    FilterPixels?: true;

    @Field(() => Boolean, {nullable:true})
    BlobPixels?: true;

    @Field(() => Boolean, {nullable:true})
    Blobs?: true;

    @Field(() => Boolean, {nullable:true})
    MinBlobSize?: true;

    @Field(() => Boolean, {nullable:true})
    MaxBlobSize?: true;

    @Field(() => Boolean, {nullable:true})
    MinX?: true;

    @Field(() => Boolean, {nullable:true})
    MaxX?: true;

    @Field(() => Boolean, {nullable:true})
    MinY?: true;

    @Field(() => Boolean, {nullable:true})
    MaxY?: true;

    @Field(() => Boolean, {nullable:true})
    Score?: true;
}
