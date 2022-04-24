import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ControlPresetsMaxAggregateInput {

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    Preset?: true;

    @Field(() => Boolean, {nullable:true})
    Label?: true;
}
