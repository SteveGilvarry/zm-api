import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Devices_Type } from '../prisma/devices-type.enum';

@ObjectType()
export class DevicesMinAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => Devices_Type, {nullable:true})
    Type?: keyof typeof Devices_Type;

    @Field(() => String, {nullable:true})
    KeyString?: string;
}
