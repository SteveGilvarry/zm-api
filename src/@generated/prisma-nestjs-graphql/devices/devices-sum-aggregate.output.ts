import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class DevicesSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;
}
