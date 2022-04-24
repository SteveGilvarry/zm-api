import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class StorageMaxAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Path?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Type?: true;

    @Field(() => Boolean, {nullable:true})
    Url?: true;

    @Field(() => Boolean, {nullable:true})
    DiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    Scheme?: true;

    @Field(() => Boolean, {nullable:true})
    ServerId?: true;

    @Field(() => Boolean, {nullable:true})
    DoDelete?: true;

    @Field(() => Boolean, {nullable:true})
    Enabled?: true;
}
