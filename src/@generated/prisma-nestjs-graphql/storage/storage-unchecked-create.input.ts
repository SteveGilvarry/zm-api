import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Storage_Type } from '../prisma/storage-type.enum';
import { Storage_Scheme } from '../prisma/storage-scheme.enum';

@InputType()
export class StorageUncheckedCreateInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Path?: string;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => Storage_Type, {nullable:true})
    Type?: keyof typeof Storage_Type;

    @Field(() => String, {nullable:true})
    Url?: string;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;

    @Field(() => Storage_Scheme, {nullable:true})
    Scheme?: keyof typeof Storage_Scheme;

    @Field(() => Int, {nullable:true})
    ServerId?: number;

    @Field(() => Boolean, {nullable:true})
    DoDelete?: boolean;

    @Field(() => Boolean, {nullable:true})
    Enabled?: boolean;
}
