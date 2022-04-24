import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class TriggersX10MinAggregateInput {

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    Activation?: true;

    @Field(() => Boolean, {nullable:true})
    AlarmInput?: true;

    @Field(() => Boolean, {nullable:true})
    AlarmOutput?: true;
}
