import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Storage_Type } from '../prisma/storage-type.enum';
import { Storage_Scheme } from '../prisma/storage-scheme.enum';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Storage {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false,defaultValue:''})
    Path!: string;

    @Field(() => String, {nullable:false,defaultValue:''})
    Name!: string;

    @Field(() => Storage_Type, {nullable:false,defaultValue:'local'})
    Type!: keyof typeof Storage_Type;

    @Field(() => String, {nullable:true})
    Url!: string | null;

    @Field(() => String, {nullable:true})
    DiskSpace!: bigint | null;

    @Field(() => Storage_Scheme, {nullable:false,defaultValue:'Medium'})
    Scheme!: keyof typeof Storage_Scheme;

    @Field(() => Int, {nullable:true})
    ServerId!: number | null;

    @Field(() => Boolean, {nullable:false,defaultValue:true})
    DoDelete!: boolean;

    @Field(() => Boolean, {nullable:false,defaultValue:true})
    Enabled!: boolean;
}
