import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class UsersCountAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Username?: true;

    @Field(() => Boolean, {nullable:true})
    Password?: true;

    @Field(() => Boolean, {nullable:true})
    Language?: true;

    @Field(() => Boolean, {nullable:true})
    Enabled?: true;

    @Field(() => Boolean, {nullable:true})
    Stream?: true;

    @Field(() => Boolean, {nullable:true})
    Events?: true;

    @Field(() => Boolean, {nullable:true})
    Control?: true;

    @Field(() => Boolean, {nullable:true})
    Monitors?: true;

    @Field(() => Boolean, {nullable:true})
    Groups?: true;

    @Field(() => Boolean, {nullable:true})
    Devices?: true;

    @Field(() => Boolean, {nullable:true})
    Snapshots?: true;

    @Field(() => Boolean, {nullable:true})
    System?: true;

    @Field(() => Boolean, {nullable:true})
    MaxBandwidth?: true;

    @Field(() => Boolean, {nullable:true})
    MonitorIds?: true;

    @Field(() => Boolean, {nullable:true})
    TokenMinExpiry?: true;

    @Field(() => Boolean, {nullable:true})
    APIEnabled?: true;

    @Field(() => Boolean, {nullable:true})
    HomeView?: true;

    @Field(() => Boolean, {nullable:true})
    _all?: true;
}
