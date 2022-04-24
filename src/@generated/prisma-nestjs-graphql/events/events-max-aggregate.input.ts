import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class EventsMaxAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    StorageId?: true;

    @Field(() => Boolean, {nullable:true})
    SecondaryStorageId?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Cause?: true;

    @Field(() => Boolean, {nullable:true})
    StartDateTime?: true;

    @Field(() => Boolean, {nullable:true})
    EndDateTime?: true;

    @Field(() => Boolean, {nullable:true})
    Width?: true;

    @Field(() => Boolean, {nullable:true})
    Height?: true;

    @Field(() => Boolean, {nullable:true})
    Length?: true;

    @Field(() => Boolean, {nullable:true})
    Frames?: true;

    @Field(() => Boolean, {nullable:true})
    AlarmFrames?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultVideo?: true;

    @Field(() => Boolean, {nullable:true})
    SaveJPEGs?: true;

    @Field(() => Boolean, {nullable:true})
    TotScore?: true;

    @Field(() => Boolean, {nullable:true})
    AvgScore?: true;

    @Field(() => Boolean, {nullable:true})
    MaxScore?: true;

    @Field(() => Boolean, {nullable:true})
    Archived?: true;

    @Field(() => Boolean, {nullable:true})
    Videoed?: true;

    @Field(() => Boolean, {nullable:true})
    Uploaded?: true;

    @Field(() => Boolean, {nullable:true})
    Emailed?: true;

    @Field(() => Boolean, {nullable:true})
    Messaged?: true;

    @Field(() => Boolean, {nullable:true})
    Executed?: true;

    @Field(() => Boolean, {nullable:true})
    Notes?: true;

    @Field(() => Boolean, {nullable:true})
    StateId?: true;

    @Field(() => Boolean, {nullable:true})
    Orientation?: true;

    @Field(() => Boolean, {nullable:true})
    DiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    Scheme?: true;

    @Field(() => Boolean, {nullable:true})
    Locked?: true;
}
