import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class GroupsUncheckedCreateInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => Int, {nullable:true})
    ParentId?: number;
}
