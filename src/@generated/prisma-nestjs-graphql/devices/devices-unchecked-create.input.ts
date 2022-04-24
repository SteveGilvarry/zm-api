import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Devices_Type } from '../prisma/devices-type.enum';

@InputType()
export class DevicesUncheckedCreateInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Devices_Type, {nullable:true})
    Type?: keyof typeof Devices_Type;

    @Field(() => String, {nullable:true})
    KeyString?: string;
}
