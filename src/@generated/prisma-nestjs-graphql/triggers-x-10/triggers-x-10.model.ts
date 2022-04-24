import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';

@ObjectType()
export class TriggersX10 {

    @Field(() => ID, {nullable:false,defaultValue:0})
    MonitorId!: number;

    @Field(() => String, {nullable:true})
    Activation!: string | null;

    @Field(() => String, {nullable:true})
    AlarmInput!: string | null;

    @Field(() => String, {nullable:true})
    AlarmOutput!: string | null;
}
