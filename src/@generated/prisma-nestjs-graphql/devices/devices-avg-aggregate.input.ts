import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class DevicesAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;
}
