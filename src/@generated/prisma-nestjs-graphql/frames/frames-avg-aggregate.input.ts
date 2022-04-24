import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class FramesAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    EventId?: true;

    @Field(() => Boolean, {nullable:true})
    FrameId?: true;

    @Field(() => Boolean, {nullable:true})
    Delta?: true;

    @Field(() => Boolean, {nullable:true})
    Score?: true;
}
