import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class GroupsAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    ParentId?: number;
}
