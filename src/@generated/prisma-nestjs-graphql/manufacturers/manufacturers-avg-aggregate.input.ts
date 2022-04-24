import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ManufacturersAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;
}
