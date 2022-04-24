import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class TriggersX10CreateInput {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => String, {nullable:true})
    Activation?: string;

    @Field(() => String, {nullable:true})
    AlarmInput?: string;

    @Field(() => String, {nullable:true})
    AlarmOutput?: string;
}
