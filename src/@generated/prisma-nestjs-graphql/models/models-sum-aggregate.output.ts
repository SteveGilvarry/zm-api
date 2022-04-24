import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ModelsSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => Int, {nullable:true})
    ManufacturerId?: number;
}
