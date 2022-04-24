import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class UsersAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    Enabled?: number;

    @Field(() => Float, {nullable:true})
    TokenMinExpiry?: number;

    @Field(() => Float, {nullable:true})
    APIEnabled?: number;
}
