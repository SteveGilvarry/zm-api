import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ZonePresetsAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    MinPixelThreshold?: true;

    @Field(() => Boolean, {nullable:true})
    MaxPixelThreshold?: true;

    @Field(() => Boolean, {nullable:true})
    MinAlarmPixels?: true;

    @Field(() => Boolean, {nullable:true})
    MaxAlarmPixels?: true;

    @Field(() => Boolean, {nullable:true})
    FilterX?: true;

    @Field(() => Boolean, {nullable:true})
    FilterY?: true;

    @Field(() => Boolean, {nullable:true})
    MinFilterPixels?: true;

    @Field(() => Boolean, {nullable:true})
    MaxFilterPixels?: true;

    @Field(() => Boolean, {nullable:true})
    MinBlobPixels?: true;

    @Field(() => Boolean, {nullable:true})
    MaxBlobPixels?: true;

    @Field(() => Boolean, {nullable:true})
    MinBlobs?: true;

    @Field(() => Boolean, {nullable:true})
    MaxBlobs?: true;

    @Field(() => Boolean, {nullable:true})
    OverloadFrames?: true;

    @Field(() => Boolean, {nullable:true})
    ExtendAlarmFrames?: true;
}
