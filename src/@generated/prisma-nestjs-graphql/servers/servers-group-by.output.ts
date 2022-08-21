import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Servers_Status } from '../prisma/servers-status.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { ServersCountAggregate } from './servers-count-aggregate.output';
import { ServersAvgAggregate } from './servers-avg-aggregate.output';
import { ServersSumAggregate } from './servers-sum-aggregate.output';
import { ServersMinAggregate } from './servers-min-aggregate.output';
import { ServersMaxAggregate } from './servers-max-aggregate.output';

@ObjectType()
export class ServersGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

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

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Int, {nullable:true})
    State_Id?: number;

    @Field(() => Servers_Status, {nullable:false})
    Status!: keyof typeof Servers_Status;

    @Field(() => GraphQLDecimal, {nullable:true})
    CpuLoad?: Decimal;

    @Field(() => String, {nullable:true})
    TotalMem?: bigint | number;

    @Field(() => String, {nullable:true})
    FreeMem?: bigint | number;

    @Field(() => String, {nullable:true})
    TotalSwap?: bigint | number;

    @Field(() => String, {nullable:true})
    FreeSwap?: bigint | number;

    @Field(() => Boolean, {nullable:false})
    zmstats!: boolean;

    @Field(() => Boolean, {nullable:false})
    zmaudit!: boolean;

    @Field(() => Boolean, {nullable:false})
    zmtrigger!: boolean;

    @Field(() => Boolean, {nullable:false})
    zmeventnotification!: boolean;

    @Field(() => ServersCountAggregate, {nullable:true})
    _count?: ServersCountAggregate;

    @Field(() => ServersAvgAggregate, {nullable:true})
    _avg?: ServersAvgAggregate;

    @Field(() => ServersSumAggregate, {nullable:true})
    _sum?: ServersSumAggregate;

    @Field(() => ServersMinAggregate, {nullable:true})
    _min?: ServersMinAggregate;

    @Field(() => ServersMaxAggregate, {nullable:true})
    _max?: ServersMaxAggregate;
}
