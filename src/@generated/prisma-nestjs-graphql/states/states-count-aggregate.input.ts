import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class StatesCountAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Definition?: true;

    @Field(() => Boolean, {nullable:true})
    IsActive?: true;

    @Field(() => Boolean, {nullable:true})
    _all?: true;
}
