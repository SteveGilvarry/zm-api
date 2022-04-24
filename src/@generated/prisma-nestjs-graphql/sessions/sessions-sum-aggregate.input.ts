import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class SessionsSumAggregateInput {

    @Field(() => Boolean, {nullable:true})
    access?: true;
}
