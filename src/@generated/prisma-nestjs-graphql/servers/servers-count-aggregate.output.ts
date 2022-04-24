import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ServersCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Protocol!: number;

    @Field(() => Int, {nullable:false})
    Hostname!: number;

    @Field(() => Int, {nullable:false})
    Port!: number;

    @Field(() => Int, {nullable:false})
    PathToIndex!: number;

    @Field(() => Int, {nullable:false})
    PathToZMS!: number;

    @Field(() => Int, {nullable:false})
    PathToApi!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    State_Id!: number;

    @Field(() => Int, {nullable:false})
    Status!: number;

    @Field(() => Int, {nullable:false})
    CpuLoad!: number;

    @Field(() => Int, {nullable:false})
    TotalMem!: number;

    @Field(() => Int, {nullable:false})
    FreeMem!: number;

    @Field(() => Int, {nullable:false})
    TotalSwap!: number;

    @Field(() => Int, {nullable:false})
    FreeSwap!: number;

    @Field(() => Int, {nullable:false})
    zmstats!: number;

    @Field(() => Int, {nullable:false})
    zmaudit!: number;

    @Field(() => Int, {nullable:false})
    zmtrigger!: number;

    @Field(() => Int, {nullable:false})
    zmeventnotification!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
