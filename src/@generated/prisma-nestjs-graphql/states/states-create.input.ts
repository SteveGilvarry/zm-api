import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class StatesCreateInput {

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => String, {nullable:false})
    Definition!: string;

    @Field(() => Int, {nullable:true})
    IsActive?: number;
}
