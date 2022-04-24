import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class ManufacturersWhereUniqueInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;
}
