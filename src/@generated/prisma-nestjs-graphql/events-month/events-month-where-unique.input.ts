import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class Events_MonthWhereUniqueInput {

    @Field(() => Int, {nullable:true})
    EventId?: number;
}
