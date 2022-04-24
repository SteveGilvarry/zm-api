import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class MontageLayoutsMaxAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Positions?: true;
}