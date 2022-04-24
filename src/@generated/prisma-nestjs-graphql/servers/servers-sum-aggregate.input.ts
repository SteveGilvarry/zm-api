import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ServersSumAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Port?: true;

    @Field(() => Boolean, {nullable:true})
    State_Id?: true;

    @Field(() => Boolean, {nullable:true})
    CpuLoad?: true;

    @Field(() => Boolean, {nullable:true})
    TotalMem?: true;

    @Field(() => Boolean, {nullable:true})
    FreeMem?: true;

    @Field(() => Boolean, {nullable:true})
    TotalSwap?: true;

    @Field(() => Boolean, {nullable:true})
    FreeSwap?: true;
}
