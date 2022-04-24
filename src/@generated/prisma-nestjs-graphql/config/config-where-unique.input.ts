import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ConfigWhereUniqueInput {

    @Field(() => String, {nullable:true})
    Name?: string;
}
