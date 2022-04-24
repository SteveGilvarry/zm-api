import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class TriggersX10SumAggregateInput {

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;
}
