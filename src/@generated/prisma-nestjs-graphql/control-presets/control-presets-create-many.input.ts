import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class ControlPresetsCreateManyInput {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => Int, {nullable:true})
    Preset?: number;

    @Field(() => String, {nullable:false})
    Label!: string;
}
