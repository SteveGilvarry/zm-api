import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class ControlsWhereUniqueInput {

    @Field(() => Int, {nullable:true})
    Id?: number;
}
