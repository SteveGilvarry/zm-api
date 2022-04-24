import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class GroupsMinAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => Int, {nullable:true})
    ParentId?: number;
}
