import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ModelsMinAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    ManufacturerId?: true;
}
