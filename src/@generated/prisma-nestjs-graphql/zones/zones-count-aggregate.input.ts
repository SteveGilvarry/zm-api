import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ZonesCountAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Type?: true;

    @Field(() => Boolean, {nullable:true})
    Units?: true;

    @Field(() => Boolean, {nullable:true})
    NumCoords?: true;

    @Field(() => Boolean, {nullable:true})
    Coords?: true;

    @Field(() => Boolean, {nullable:true})
    Area?: true;

    @Field(() => Boolean, {nullable:true})
    AlarmRGB?: true;

    @Field(() => Boolean, {nullable:true})
    CheckMethod?: true;

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

    @Field(() => Boolean, {nullable:true})
    _all?: true;
}
