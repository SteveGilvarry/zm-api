import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Devices_Type } from '../prisma/devices-type.enum';

@ObjectType()
export class Devices {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Devices_Type, {nullable:false,defaultValue:'X10'})
    Type!: keyof typeof Devices_Type;

    @Field(() => String, {nullable:false,defaultValue:''})
    KeyString!: string;
}
