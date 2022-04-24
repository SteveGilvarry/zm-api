import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class Monitor_StatusWhereUniqueInput {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;
}
