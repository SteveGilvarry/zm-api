import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class DevicesSumAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;
}
