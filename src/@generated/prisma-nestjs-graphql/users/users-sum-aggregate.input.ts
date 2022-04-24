import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class UsersSumAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Enabled?: true;

    @Field(() => Boolean, {nullable:true})
    TokenMinExpiry?: true;

    @Field(() => Boolean, {nullable:true})
    APIEnabled?: true;
}
