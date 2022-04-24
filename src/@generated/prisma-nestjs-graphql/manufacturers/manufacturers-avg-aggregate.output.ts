import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class ManufacturersAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;
}
