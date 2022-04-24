import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';

@ObjectType()
export class Manufacturers {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;
}
