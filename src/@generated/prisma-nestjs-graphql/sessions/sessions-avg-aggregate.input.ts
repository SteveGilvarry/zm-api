import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class SessionsAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    access?: true;
}
