import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class StatesSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    IsActive?: number;
}
