import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class DevicesAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;
}
