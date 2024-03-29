import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Servers_Status } from '../prisma/servers-status.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { transformToDecimal } from 'prisma-graphql-type-decimal';
import { Transform } from 'class-transformer';
import { Type } from 'class-transformer';

@InputType()
export class ServersUncheckedCreateInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Protocol?: string;

    @Field(() => String, {nullable:true})
    Hostname?: string;

    @Field(() => Int, {nullable:true})
    Port?: number;

    @Field(() => String, {nullable:true})
    PathToIndex?: string;

    @Field(() => String, {nullable:true})
    PathToZMS?: string;

    @Field(() => String, {nullable:true})
    PathToApi?: string;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => Int, {nullable:true})
    State_Id?: number;

    @Field(() => Servers_Status, {nullable:true})
    Status?: keyof typeof Servers_Status;

    @Field(() => GraphQLDecimal, {nullable:true})
    @Type(() => Object)
    @Transform(transformToDecimal)
    CpuLoad?: Decimal;

    @Field(() => String, {nullable:true})
    TotalMem?: bigint | number;

    @Field(() => String, {nullable:true})
    FreeMem?: bigint | number;

    @Field(() => String, {nullable:true})
    TotalSwap?: bigint | number;

    @Field(() => String, {nullable:true})
    FreeSwap?: bigint | number;

    @Field(() => Boolean, {nullable:true})
    zmstats?: boolean;

    @Field(() => Boolean, {nullable:true})
    zmaudit?: boolean;

    @Field(() => Boolean, {nullable:true})
    zmtrigger?: boolean;

    @Field(() => Boolean, {nullable:true})
    zmeventnotification?: boolean;
}
