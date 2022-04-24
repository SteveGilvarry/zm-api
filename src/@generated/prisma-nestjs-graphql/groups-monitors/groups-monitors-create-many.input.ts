import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class Groups_MonitorsCreateManyInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:false})
    GroupId!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;
}
