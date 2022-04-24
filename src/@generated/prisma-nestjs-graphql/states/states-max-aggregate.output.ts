import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class StatesMaxAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => String, {nullable:true})
    Definition?: string;

    @Field(() => Int, {nullable:true})
    IsActive?: number;
}
