import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class SessionsAvgAggregate {

    @Field(() => Float, {nullable:true})
    access?: number;
}
