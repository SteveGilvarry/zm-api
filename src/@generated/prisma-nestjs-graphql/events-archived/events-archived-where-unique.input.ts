import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class Events_ArchivedWhereUniqueInput {

    @Field(() => Int, {nullable:true})
    EventId?: number;
}
