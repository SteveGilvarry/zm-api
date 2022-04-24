import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ControlPresets {

    @Field(() => Int, {nullable:false,defaultValue:0})
    MonitorId!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Preset!: number;

    @Field(() => String, {nullable:false})
    Label!: string;
}
