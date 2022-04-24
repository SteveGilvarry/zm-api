import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class UsersSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    Enabled?: number;

    @Field(() => String, {nullable:true})
    TokenMinExpiry?: bigint | number;

    @Field(() => Int, {nullable:true})
    APIEnabled?: number;
}
