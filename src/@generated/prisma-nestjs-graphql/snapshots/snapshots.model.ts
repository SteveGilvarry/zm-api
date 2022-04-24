import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Snapshots {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:true})
    Name!: string | null;

    @Field(() => String, {nullable:true})
    Description!: string | null;

    @Field(() => Int, {nullable:true})
    CreatedBy!: number | null;

    @Field(() => Date, {nullable:true})
    CreatedOn!: Date | null;
}
