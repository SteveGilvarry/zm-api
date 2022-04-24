import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class UsersCountOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Username?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Password?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Language?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Enabled?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Stream?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Events?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Control?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Monitors?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Groups?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Devices?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Snapshots?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    System?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxBandwidth?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorIds?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TokenMinExpiry?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    APIEnabled?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    HomeView?: keyof typeof SortOrder;
}
