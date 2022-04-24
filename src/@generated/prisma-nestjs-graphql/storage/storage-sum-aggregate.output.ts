import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class StorageSumAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;

    @Field(() => Int, {nullable:true})
    ServerId?: number;
}
