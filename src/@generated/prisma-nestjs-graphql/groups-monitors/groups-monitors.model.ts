import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Groups_Monitors {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    GroupId!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;
}
