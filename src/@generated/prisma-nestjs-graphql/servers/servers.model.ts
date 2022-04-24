import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Servers_Status } from '../prisma/servers-status.enum';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@ObjectType()
export class Servers {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:true})
    Protocol!: string | null;

    @Field(() => String, {nullable:true})
    Hostname!: string | null;

    @Field(() => Int, {nullable:true})
    Port!: number | null;

    @Field(() => String, {nullable:true})
    PathToIndex!: string | null;

    @Field(() => String, {nullable:true})
    PathToZMS!: string | null;

    @Field(() => String, {nullable:true})
    PathToApi!: string | null;

    @Field(() => String, {nullable:false,defaultValue:''})
    Name!: string;

    @Field(() => Int, {nullable:true})
    State_Id!: number | null;

    @Field(() => Servers_Status, {nullable:false,defaultValue:'Unknown'})
    Status!: keyof typeof Servers_Status;

    @Field(() => GraphQLDecimal, {nullable:true})
    CpuLoad!: any | null;

    @Field(() => String, {nullable:true})
    TotalMem!: bigint | null;

    @Field(() => String, {nullable:true})
    FreeMem!: bigint | null;

    @Field(() => String, {nullable:true})
    TotalSwap!: bigint | null;

    @Field(() => String, {nullable:true})
    FreeSwap!: bigint | null;

    @Field(() => Boolean, {nullable:false,defaultValue:false})
    zmstats!: boolean;

    @Field(() => Boolean, {nullable:false,defaultValue:false})
    zmaudit!: boolean;

    @Field(() => Boolean, {nullable:false,defaultValue:false})
    zmtrigger!: boolean;

    @Field(() => Boolean, {nullable:false,defaultValue:false})
    zmeventnotification!: boolean;
}
