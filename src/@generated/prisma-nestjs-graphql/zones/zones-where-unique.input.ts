import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class ZonesWhereUniqueInput {

    @Field(() => Int, {nullable:true})
    Id?: number;
}