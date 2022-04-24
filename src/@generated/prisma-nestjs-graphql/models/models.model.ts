import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Models {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Int, {nullable:true})
    ManufacturerId!: number | null;
}
