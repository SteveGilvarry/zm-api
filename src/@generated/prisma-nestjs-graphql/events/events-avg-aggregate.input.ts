import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class EventsAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    StorageId?: true;

    @Field(() => Boolean, {nullable:true})
    SecondaryStorageId?: true;

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
    StateId?: true;

    @Field(() => Boolean, {nullable:true})
    DiskSpace?: true;
}
