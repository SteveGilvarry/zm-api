import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class SnapshotsCountAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Description?: true;

    @Field(() => Boolean, {nullable:true})
    CreatedBy?: true;

    @Field(() => Boolean, {nullable:true})
    CreatedOn?: true;

    @Field(() => Boolean, {nullable:true})
    _all?: true;
}
