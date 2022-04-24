import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class SessionsSumAggregate {

    @Field(() => Int, {nullable:true})
    access?: number;
}
