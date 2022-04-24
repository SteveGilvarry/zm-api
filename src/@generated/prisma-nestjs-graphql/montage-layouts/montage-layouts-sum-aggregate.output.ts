import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class MontageLayoutsSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;
}
