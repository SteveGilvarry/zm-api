import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class GroupsMinAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    ParentId?: true;
}
