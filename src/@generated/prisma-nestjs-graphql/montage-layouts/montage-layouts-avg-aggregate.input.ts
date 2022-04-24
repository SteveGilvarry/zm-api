import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class MontageLayoutsAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;
}
