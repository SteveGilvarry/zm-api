import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class Groups_MonitorsMinAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    GroupId?: true;

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;
}
