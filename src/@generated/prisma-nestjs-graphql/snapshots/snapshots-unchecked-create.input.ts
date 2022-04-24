import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class SnapshotsUncheckedCreateInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => String, {nullable:true})
    Description?: string;

    @Field(() => Int, {nullable:true})
    CreatedBy?: number;

    @Field(() => Date, {nullable:true})
    CreatedOn?: Date | string;
}
