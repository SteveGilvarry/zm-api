import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class Event_SummariesWhereUniqueInput {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;
}
