import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class StatesAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    IsActive?: number;
}
