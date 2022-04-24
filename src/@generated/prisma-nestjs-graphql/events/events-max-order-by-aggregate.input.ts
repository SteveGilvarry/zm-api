import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class EventsMaxOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StorageId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SecondaryStorageId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Cause?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StartDateTime?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    EndDateTime?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Width?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Height?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Length?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Frames?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmFrames?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultVideo?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SaveJPEGs?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotScore?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AvgScore?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxScore?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Archived?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Videoed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Uploaded?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Emailed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Messaged?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Executed?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Notes?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    StateId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Orientation?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Scheme?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Locked?: keyof typeof SortOrder;
}
