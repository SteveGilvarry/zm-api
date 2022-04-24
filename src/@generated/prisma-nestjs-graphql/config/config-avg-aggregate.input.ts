import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ConfigAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Readonly?: true;
}
