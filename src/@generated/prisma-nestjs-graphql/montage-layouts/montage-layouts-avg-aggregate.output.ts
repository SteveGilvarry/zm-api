import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class MontageLayoutsAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;
}
